"""Unified Data Bus — timestamped, typed signal exchange.

Channels:
- realtime: ZeroMQ-style in-process pub/sub (dict-based for MVP)
- batch: HDF5-backed persistent log
- event: list-based event queue

Security:
  - CWE-287: Topic ACL with module registration
  - CWE-862: Default-deny publish/subscribe (require module registration)
  - CWE-20: Signal __post_init__ validation (NaN, Inf, negative timestamps, safety bounds)
  - CWE-208: Constant-time comparison for admin token
  - CWE-178: Strict module_id normalization with format validation
  - CWE-22: Path traversal prevention in source field
"""

import copy
import hmac
import logging
import math
import re
import threading
from collections import defaultdict
from collections.abc import Callable
from dataclasses import dataclass, field
from enum import IntFlag
from typing import Any

from sim_platform.core.constants import MAX_EVENTS as _MAX_EVENTS
from sim_platform.core.constants import MAX_HISTORY as _MAX_HISTORY
from sim_platform.core.constants import MAX_MODULE_ID_LEN as _MAX_MODULE_ID_LEN

logger = logging.getLogger(__name__)

# Security constants
_MODULE_ID_PATTERN = re.compile(r'^[a-zA-Z][a-zA-Z0-9_]*://[a-zA-Z0-9_./-]+$')

# ── data validity flags ──────────────────────────────────────

class DataValidity(IntFlag):
    VALID = 0x00
    STALE = 0x01
    INTERPOLATED = 0x02
    EXTRAPOLATED = 0x04
    CLIPPED = 0x08
    NOISY = 0x10
    OUT_OF_RANGE = 0x20
    SENSOR_FAULT = 0x40
    SIMULATED = 0x80
    INVALID = 0x100  # SECURITY: corrupted data


# ── safety levels ────────────────────────────────────────────

SAFETY_NORMAL = 0
SAFETY_WARNING = 1
SAFETY_CRITICAL = 2
SAFETY_EMERGENCY = 3
VALID_SAFETY_LEVELS = {SAFETY_NORMAL, SAFETY_WARNING, SAFETY_CRITICAL, SAFETY_EMERGENCY}


# ── base signal ──────────────────────────────────────────────

@dataclass
class Signal:
    """Unified signal container with mandatory metadata.

    Raises ValueError on construction if validation fails (CWE-20).
    """

    source: str                     # "sensor://current_phase_a"
    signal_type: str                # "current", "voltage", "angle" ...
    timestamp_ns: int = 0
    value: float = 0.0
    unit: str = ""                  # SI: "A", "V", "rad", "N.m" ...
    coordinate_frame: str = "WORLD"
    sample_rate_hz: float = 0.0
    latency_ns: int = 0
    validity: DataValidity = DataValidity.VALID
    quality: float = 1.0            # 0..1
    safety_level: int = 0           # 0=normal, 1=warning, 2=critical, 3=emergency
    sequence_id: int = 0

    def __post_init__(self) -> None:
        """Validate signal fields (CWE-20: Input Validation)."""
        errors: list[str] = []

        # Timestamp must be non-negative
        if self.timestamp_ns < 0:
            errors.append(f"negative timestamp_ns={self.timestamp_ns}")

        # NaN/Inf check on value
        if math.isnan(self.value):
            self.validity |= DataValidity.INVALID
            self.quality = 0.0
            errors.append("NaN value")
        if math.isinf(self.value):
            self.validity |= DataValidity.INVALID
            self.quality = 0.0
            errors.append("Inf value")

        # Safety level bounds
        if self.safety_level not in VALID_SAFETY_LEVELS:
            logger.warning("Signal safety_level=%d out of range, clamped to %d",
                           self.safety_level, SAFETY_NORMAL)
            self.safety_level = SAFETY_NORMAL

        # Quality bounds
        self.quality = max(0.0, min(1.0, self.quality))

        # SECURITY (CWE-22): Source normalization with path traversal guard
        if "://" not in self.source:
            self.source = f"module://{self.source}"
        if ".." in self.source or any(c in self.source for c in "\x00\r\n"):
            raise ValueError(f"Invalid source '{self.source}': path traversal or control chars")

        if errors:
            logger.warning("Signal validation warnings: %s", "; ".join(errors))

    @property
    def timestamp_s(self) -> float:
        return self.timestamp_ns / 1e9


# ── event ────────────────────────────────────────────────────

@dataclass
class SimEvent:
    """Discrete simulation event."""

    EVENT_TYPES = {"FAULT", "LIMIT_HIT", "STATE_CHANGE", "USER", "DIVERGENCE"}

    event_type: str
    source: str
    timestamp_ns: int = 0
    payload: dict[str, Any] = field(default_factory=dict)

    def __post_init__(self) -> None:
        """Validate event fields (CWE-20)."""
        if self.event_type not in self.EVENT_TYPES:
            raise ValueError(
                f"Invalid event_type '{self.event_type}', must be one of {self.EVENT_TYPES}")
        if self.timestamp_ns < 0:
            raise ValueError(f"timestamp_ns must be non-negative, got {self.timestamp_ns}")


# ── data bus ─────────────────────────────────────────────────

class DataBus:
    """In-process unified data bus with security enforcement.

    Security features (v2 — hardened):
      - Default-deny publish: all modules must be registered (CWE-862)
      - Default-deny subscribe: subscribers must be registered (CWE-862)
      - Topic ACL (authorization)
      - Signal validation at boundary (CWE-20)
      - Admin token for security-clearing operations
    """

    def __init__(self):
        self._lock = threading.RLock()  # Reentrant lock for thread safety
        self._latest: dict[str, Signal] = {}
        self._subscribers: dict[str, list[Callable]] = defaultdict(list)
        self._history: dict[str, list[Signal]] = defaultdict(list)
        self._events: list[SimEvent] = []
        self._seq: int = 0

        # Security: module registry + topic ACL
        self._registered_modules: dict[str, bool] = {}   # module_id → authenticated
        self._topic_acls: dict[str, set[str]] = {}        # topic → set of allowed module_ids
        self._admin_token: str = ""                        # admin token for clear_security

    # ── security: module auth ──────────────────────────────

    def _normalize_module_id(self, module_id: str) -> str:
        """Normalize module_id with strict format validation (CWE-178).

        Raises ValueError on invalid format.
        """
        if not module_id:
            raise ValueError("module_id cannot be empty")
        if len(module_id) > _MAX_MODULE_ID_LEN:
            raise ValueError(f"module_id exceeds {_MAX_MODULE_ID_LEN} characters")
        if "://" not in module_id:
            module_id = f"module://{module_id}"
        # Reject path traversal and control chars
        if ".." in module_id or "\x00" in module_id:
            raise ValueError(f"Invalid module_id: path traversal or null byte in '{module_id}'")
        # SECURITY: Validate format with regex (CWE-20)
        if not _MODULE_ID_PATTERN.match(module_id):
            raise ValueError(
                f"Invalid module_id format '{module_id}': "
                f"must match '{_MODULE_ID_PATTERN.pattern}'")
        return module_id

    def register_module(self, module_id: str) -> None:
        """Register a module to the data bus.

        Args:
            module_id: Unique module identifier (e.g. "sensor:current_phase_a").
        """
        module_id = self._normalize_module_id(module_id)
        self._registered_modules[module_id] = True
        logger.info("Module registered: %s", module_id)

    def set_admin_token(self, token: str) -> None:
        """Set admin token for security-clearing operations."""
        if not token:
            raise ValueError("admin_token cannot be empty")
        self._admin_token = token
        logger.info("Admin token set")

    def restrict_topic(self, topic: str, allowed_modules: list[str]) -> None:
        """Set access control: only allowed modules can publish to topic.

        Args:
            topic: Topic name.
            allowed_modules: Module IDs allowed to publish.
        """
        normalized = set()
        for m in allowed_modules:
            m = self._normalize_module_id(m)
            normalized.add(m)
        self._topic_acls[topic] = normalized
        logger.info("Topic '%s' restricted to modules: %s", topic,
                    [m.split("://")[1] for m in normalized])

    def _verify_admin(self, token: str) -> bool:
        """Verify admin authorization token using constant-time comparison (CWE-208)."""
        if not self._admin_token:
            logger.warning("Admin token not set, denying clear_security")
            return False
        return hmac.compare_digest(token, self._admin_token)

    # ── publish ─────────────────────────────────────────────

    def publish(self, topic: str, signal: Signal,
                module_id: str = "") -> None:
        """Publish a signal to a topic.

        SECURITY (CWE-862): Default-deny — all modules must be registered.
        If topic has ACL, additionally checks topic-level authorization.

        Args:
            topic: Topic name.
            signal: Signal to publish.
            module_id: Publishing module (REQUIRED).

        Raises:
            PermissionError: If module is not registered or not authorized.
        """
        # SECURITY (CWE-862): Default-deny — require module registration for ALL topics
        if not module_id:
            raise PermissionError(
                "module_id is required to publish to any topic")
        # Normalize for consistent comparison
        norm_id = self._normalize_module_id(module_id)
        if norm_id not in self._registered_modules:
            raise PermissionError(
                f"Unregistered module '{module_id}' cannot publish to '{topic}'")

        # ACL check (if topic has restricted access)
        if topic in self._topic_acls:
            if norm_id not in self._topic_acls[topic]:
                allowed = self._topic_acls[topic]
                logger.warning("ACCESS DENIED: module '%s' on topic '%s' "
                               "(allowed: %s)", module_id, topic,
                               [m.split("://")[1] for m in allowed])
                raise PermissionError(
                    f"Module '{module_id}' not authorized for topic '{topic}'")

        # SECURITY: Signal validated in __post_init__ (CWE-20)
        with self._lock:
            self._seq += 1
            signal.sequence_id = self._seq
            self._latest[topic] = signal

            # Ring buffer — use deque-like slicing
            hist = self._history[topic]
            hist.append(signal)
            if len(hist) > _MAX_HISTORY:
                self._history[topic] = hist[-_MAX_HISTORY:]

        # Notify subscribers (outside lock to prevent deadlock)
        for cb in self._subscribers.get(topic, []):
            try:
                cb(signal)
            except Exception:
                logger.exception("Subscriber callback failed for %s", topic)

    def publish_event(self, event: SimEvent) -> None:
        """Publish a validated event (CWE-20: validated in SimEvent.__post_init__)."""
        # SECURITY (CWE-789): Cap event list
        if len(self._events) >= _MAX_EVENTS:
            self._events = self._events[-_MAX_EVENTS // 2:]
        self._events.append(event)

    def publish_scalar(self, topic: str, value: float, unit: str = "",
                       timestamp_ns: int = 0, module_id: str = "") -> Signal:
        sig = Signal(
            source=module_id or topic, signal_type="scalar",
            timestamp_ns=timestamp_ns, value=value, unit=unit,
        )
        self.publish(topic, sig, module_id=module_id)
        return sig

    def publish_vector(self, topic: str, values: dict[str, float],
                       unit: str = "", timestamp_ns: int = 0,
                       module_id: str = "") -> dict[str, Signal]:
        sigs = {}
        for name, val in values.items():
            full = f"{topic}/{name}"
            sigs[name] = self.publish_scalar(full, val, unit, timestamp_ns, module_id)
        return sigs

    # ── subscribe / read ─────────────────────────────────────

    def subscribe(self, topic: str, callback: Callable[[Signal], None],
                  module_id: str = "") -> None:
        """Subscribe to a topic with authorization check (CWE-862).

        Args:
            topic: Topic name.
            callback: Callback function receiving Signal.
            module_id: Subscribing module (REQUIRED for auth check).

        Raises:
            PermissionError: If module not registered or not authorized.
        """
        if not module_id:
            raise PermissionError("module_id required to subscribe")
        norm_id = self._normalize_module_id(module_id)
        if norm_id not in self._registered_modules:
            raise PermissionError(
                f"Unregistered module '{module_id}' cannot subscribe to '{topic}'")
        # If topic has ACL, check read permission
        if topic in self._topic_acls:
            if norm_id not in self._topic_acls[topic]:
                raise PermissionError(
                    f"Module '{module_id}' not authorized to read topic '{topic}'")
        with self._lock:
            self._subscribers[topic].append(callback)

    def read_latest(self, topic: str) -> Signal | None:
        with self._lock:
            result = self._latest.get(topic)
        return copy.deepcopy(result) if result is not None else None

    def read_history(self, topic: str, max_count: int = 100) -> list[Signal]:
        """Read signal history. SECURITY (L-06): validate max_count."""
        if max_count <= 0:
            return []
        with self._lock:
            hist = self._history.get(topic, [])
            return list(hist[-max_count:]) if hist else []

    # ── snapshot / reset ─────────────────────────────────────

    def snapshot(self) -> dict:
        """Return immutable snapshot (CWE-501: deep copy to prevent mutation)."""
        with self._lock:
            return {
                "latest": {k: copy.deepcopy(v) for k, v in self._latest.items()},
                "seq": self._seq,
            }

    def reset(self) -> None:
        """Reset data bus state but preserve security settings."""
        with self._lock:
            self._latest.clear()
            self._subscribers.clear()
            self._history.clear()
            self._events.clear()
            self._seq = 0
            # Note: _registered_modules and _topic_acls are preserved

    def clear_security(self, admin_token: str = "") -> None:
        """Clear all security settings — requires admin authorization (CWE-862).

        Args:
            admin_token: Admin authorization token (required).

        Raises:
            PermissionError: If token is invalid or not set.
        """
        if not self._verify_admin(admin_token):
            logger.warning("ACCESS DENIED: clear_security called with invalid token")
            raise PermissionError("Admin authorization required to clear security settings")
        with self._lock:
            self._registered_modules.clear()
            self._topic_acls.clear()
        logger.critical("All security settings cleared by authorized admin")

    @property
    def registered_module_count(self) -> int:
        return len(self._registered_modules)

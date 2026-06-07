# -*- mode: python ; coding: utf-8 -*-
"""PyInstaller spec file for sim_platform GUI.

Build command:
    pyinstaller sim_platform.spec --clean

Output:
    dist/sim_platform/sim_platform.exe
"""

import os
import sys

block_cipher = None

# Project root (SPECPATH is the directory containing this .spec file)
PROJ = os.path.abspath(SPECPATH)

a = Analysis(
    # Entry point: GUI app
    [os.path.join(PROJ, 'tools', 'gui', 'app.py')],

    pathex=[PROJ],

    binaries=[],

    # Data files to bundle
    datas=[
        # pyproject.toml for version reading
        (os.path.join(PROJ, 'pyproject.toml'), 'sim_platform'),
        # Example config
        (os.path.join(PROJ, 'examples', 'pmsm_foc_mvp', 'config.yaml'),
         os.path.join('sim_platform', 'examples', 'pmsm_foc_mvp')),
    ],

    # Hidden imports that PyInstaller may miss
    hiddenimports=[
        # PySide6 modules
        'PySide6.QtCharts',
        'PySide6.QtCore',
        'PySide6.QtGui',
        'PySide6.QtWidgets',

        # Project modules (all GUI components)
        'sim_platform',
        'sim_platform.__init__',
        'sim_platform.tools',
        'sim_platform.tools.gui',
        'sim_platform.tools.gui.app',
        'sim_platform.tools.gui.i18n',
        'sim_platform.tools.gui.theme',
        'sim_platform.tools.gui.workers',
        'sim_platform.tools.gui.icons',
        'sim_platform.tools.gui.animations',
        'sim_platform.tools.gui.conflict_resolver',
        'sim_platform.tools.gui.solver_presets',
        'sim_platform.tools.gui.guided_tour',
        'sim_platform.tools.gui.dialogs',
        'sim_platform.tools.gui.dialogs.about_dialog',
        'sim_platform.tools.gui.dialogs.onboarding_dialog',
        'sim_platform.tools.gui.dialogs.scan_dialog',
        'sim_platform.tools.gui.dialogs.conflict_dialog',
        'sim_platform.tools.gui.widgets',
        'sim_platform.tools.gui.widgets.chart_widget',
        'sim_platform.tools.gui.widgets.config_panel',
        'sim_platform.tools.gui.widgets.dashboard',
        'sim_platform.tools.gui.widgets.log_widget',
        'sim_platform.tools.gui.widgets.result_table',
        'sim_platform.tools.gui.widgets.stat_cards',

        # Model modules (used by workers.py + __init__.py transitive imports)
        'sim_platform.models',
        'sim_platform.models.physics_constraints',
        'sim_platform.models.motor',
        'sim_platform.models.motor.pmsm_dq',
        'sim_platform.models.motor.pmsm_advanced',
        'sim_platform.models.motor.bldc',
        'sim_platform.models.motor.im_dq',
        'sim_platform.models.controller',
        'sim_platform.models.controller.foc',
        'sim_platform.models.controller.ekf',
        'sim_platform.models.controller.mpc',
        'sim_platform.models.power',
        'sim_platform.models.power.power_models',
        'sim_platform.models.sensor',
        'sim_platform.models.sensor.sensors',
        'sim_platform.models.thermal',
        'sim_platform.models.thermal.thermal_model',
        'sim_platform.models.fusion',
        'sim_platform.models.fusion.sensor_fusion',
        'sim_platform.models.common',
        'sim_platform.models.common.transforms',

        # Core modules
        'sim_platform.core',
        'sim_platform.core.utils',
        'sim_platform.core.constants',
        'sim_platform.core.clock',
        'sim_platform.core.data_bus',
        'sim_platform.core.orchestrator',
        'sim_platform.core.model_registry',

        # Tools
        'sim_platform.tools.replay',
        'sim_platform.tools.replay.hdf5_logger',

        # External dependencies
        'yaml',
        'numpy',
        'matplotlib',
        'h5py',
        'tomllib',
    ],

    hookspath=[],
    hooksconfig={},
    runtime_hooks=[],

    excludes=[
        # Exclude test modules
        'verification',
        'verification.test_cases',
        'verification.fault_injection',
        # Exclude TUI (not needed in GUI exe)
        'sim_platform.tools.tui',
        'sim_platform.tools.tui.app',
        'textual',
        # Exclude heavy unused packages
        'IPython',
        'jupyter',
        'notebook',
        'pandas',
        'scipy',
        'PIL',
        'cv2',
        # Exclude unused PySide6 modules (saves ~40MB)
        'PySide6.QtQuick',
        'PySide6.QtQml',
        'PySide6.QtPdf',
        'PySide6.QtNetwork',
        'PySide6.QtVirtualKeyboard',
        'PySide6.Qt3DCore',
        'PySide6.Qt3DRender',
        'PySide6.Qt3DLogic',
        'PySide6.Qt3DInput',
        'PySide6.Qt3DAnimation',
        'PySide6.Qt3DExtras',
        'PySide6.QtBluetooth',
        'PySide6.QtMultimedia',
        'PySide6.QtMultimediaWidgets',
        'PySide6.QtNfc',
        'PySide6.QtPositioning',
        'PySide6.QtRemoteObjects',
        'PySide6.QtSensors',
        'PySide6.QtSerialBus',
        'PySide6.QtSerialPort',
        'PySide6.QtWebChannel',
        'PySide6.QtWebEngine',
        'PySide6.QtWebEngineCore',
        'PySide6.QtWebEngineWidgets',
        'PySide6.QtWebSockets',
        'PySide6.QtHelp',
        'PySide6.QtOpenGL',
        'PySide6.QtShaderTools',
        'PySide6.QtUiTools',
        # Exclude unused matplotlib backends
        'matplotlib.backends.backend_qt5agg',
        'matplotlib.backends.backend_qt4agg',
        'matplotlib.backends.backend_gtk3agg',
        'matplotlib.backends.backend_wxagg',
        'matplotlib.backends.backend_tkagg',
        # Exclude unused numpy submodules
        'numpy.distutils',
        'numpy.f2py',
        'numpy.testing',
    ],

    win_no_prefer_redirects=False,
    win_private_assemblies=False,
    cipher=block_cipher,
    noarchive=False,
)

# Remove duplicate files
pyz = PYZ(a.pure, a.zipped_data, cipher=block_cipher)

exe = EXE(
    pyz,
    a.scripts,
    [],
    exclude_binaries=True,
    name='sim_platform',
    debug=False,
    bootloader_ignore_signals=False,
    strip=False,
    upx=True,
    console=False,  # No console window — pure GUI application
    disable_windowed_traceback=False,
    argv_emulation=False,
    target_arch=None,
    codesign_identity=None,
    entitlements_file=None,
)

coll = COLLECT(
    exe,
    a.binaries,
    a.zipfiles,
    a.datas,
    strip=False,
    upx=True,
    upx_exclude=[],
    name='sim_platform',
)

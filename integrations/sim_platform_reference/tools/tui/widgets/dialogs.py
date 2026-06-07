"""Modal dialogs for TUI."""

from textual.app import ComposeResult
from textual.containers import Grid, Horizontal
from textual.screen import ModalScreen
from textual.widgets import Button, Static


class ErrorDialog(ModalScreen):
    """Modal error dialog with recovery options."""

    # Use hardcoded Catppuccin Mocha colors (self-contained, no variable dependencies)
    DEFAULT_CSS = """
    ErrorDialog {
        align: center middle;
    }
    #error-dialog {
        grid-size: 1;
        grid-gutter: 1;
        padding: 2 4;
        width: 60;
        height: auto;
        background: #313244;
        border: thick #F38BA8;
    }
    #err-title {
        text-style: bold;
        text-align: center;
        padding: 0 0 1 0;
        color: #F38BA8;
    }
    #err-msg {
        text-align: center;
        color: #CDD6F4;
    }
    #err-detail {
        color: #7F849C;
        text-align: center;
    }
    """

    def __init__(self, title: str, message: str, detail: str = ""):
        super().__init__()
        self._err_title = title
        self._err_msg = message
        self._err_detail = detail

    def compose(self) -> ComposeResult:
        yield Grid(
            Static(f"[bold red]\u2717 {self._err_title}[/]", id="err-title"),
            Static(self._err_msg, id="err-msg"),
            Static(self._err_detail, id="err-detail") if self._err_detail else Static(""),
            Horizontal(
                Button("Dismiss", variant="default", id="dismiss"),
                Button("Return to Main", variant="primary", id="return-home"),
                classes="button-row",
            ),
            id="error-dialog",
        )

    def on_button_pressed(self, event: Button.Pressed) -> None:
        if event.button.id == "dismiss":
            self.dismiss()
        elif event.button.id == "return-home":
            self.app.pop_screen()
            from ..screens.main import MainScreen
            self.app.goto(MainScreen)


class ConfirmDialog(ModalScreen):
    """Confirm action dialog."""

    # Use hardcoded Catppuccin Mocha colors (self-contained, no variable dependencies)
    DEFAULT_CSS = """
    ConfirmDialog {
        align: center middle;
    }
    #confirm-dialog {
        grid-size: 1;
        grid-gutter: 1;
        padding: 2 4;
        width: 60;
        height: auto;
        background: #313244;
        border: thick #F9E2AF;
    }
    #confirm-title {
        text-style: bold;
        text-align: center;
        padding: 0 0 1 0;
        color: #F9E2AF;
    }
    #confirm-msg {
        text-align: center;
        color: #CDD6F4;
    }
    """

    def __init__(self, title: str, message: str, confirm_text: str = "Confirm",
                 cancel_text: str = "Cancel", danger: bool = False):
        super().__init__()
        self._title = title
        self._msg = message
        self._confirm = confirm_text
        self._cancel = cancel_text
        self._danger = danger

    def compose(self) -> ComposeResult:
        yield Grid(
            Static(f"[bold yellow]? {self._title}[/]", id="confirm-title"),
            Static(self._msg, id="confirm-msg"),
            Horizontal(
                Button(self._cancel, variant="default", id="cancel"),
                Button(self._confirm, variant="error" if self._danger else "primary", id="confirm"),
                classes="button-row",
            ),
            id="confirm-dialog",
        )

    def on_button_pressed(self, event: Button.Pressed) -> None:
        if event.button.id == "confirm":
            self.dismiss(True)
        else:
            self.dismiss(False)

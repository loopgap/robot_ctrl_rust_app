#!/usr/bin/env python3
"""sim_platform — Multi-Domain Co-Simulation Platform.

Quick start:
    python -m sim_platform                  # Interactive runner
    python -m sim_platform --gui            # PySide6 GUI (recommended)
    python -m sim_platform --tui            # Textual TUI
    python -m sim_platform --quick          # Quick demo
    python -m sim_platform --param-scan     # Parameter scanning
    python -m sim_platform --help           # All options

TUI keyboard shortcuts:
    R=Run  C=Config  S=Scan  Esc=Back  Q=Quit  Ctrl+H=Home
"""

if __name__ == "__main__":
    import os
    import sys

    PROJ_ROOT = os.path.dirname(os.path.abspath(__file__))
    sys.path.insert(0, os.path.dirname(PROJ_ROOT))

    args = sys.argv[1:]

    if not args or args[0] in ("-i", "--interactive", "run"):
        # Launch interactive runner
        from sim_platform.tools.visualization.interactive_runner import main
        main()

    elif args[0] in ("-g", "--gui"):
        # Launch PySide6 GUI
        from sim_platform.tools.gui.app import run_app
        run_app()

    elif args[0] in ("-t", "--tui"):
        # Launch Textual TUI
        from sim_platform.tools.tui import main
        main()

    elif args[0] in ("-q", "--quick"):
        # Quick default demo
        from sim_platform.tools.visualization.interactive_runner import step3_run
        cfg = {
            "motor_params": {"Rs": 0.1, "Ld": 0.5e-3, "Lq": 1.0e-3,
                             "flux_pm": 0.03, "J": 0.001, "B": 0.0001, "Pp": 4},
            "foc": {"kp_id": 5.0, "ki_id": 500.0, "kp_iq": 5.0, "ki_iq": 500.0},
            "speed_pi": {"kp": 0.05, "ki": 0.5},
            "duration_s": 1.5,
            "speed_ref_value": 100.0,
            "profile": "step",
            "load_torque": 0.0,
            "fault_sag": False,
        }
        step3_run(cfg)

    elif args[0] in ("-s", "--param-scan"):
        from sim_platform.tools.visualization.parameter_scan import main
        sys.argv = ["parameter_scan.py"] + args[1:]
        main()

    elif args[0] in ("-p", "--plot"):
        # Plot existing HDF5
        from sim_platform.tools.replay.hdf5_logger import HDF5Logger
        from sim_platform.tools.visualization.plot_log import plot_foc_results
        if len(args) < 2:
            print("Usage: python -m sim_platform --plot <hdf5_file.h5>")
            sys.exit(1)
        filepath = args[1]
        log = HDF5Logger(filepath, "r")
        log.open()
        data = {k: log.read(k).tolist() for k in log.keys()
                if isinstance(log.read(k).tolist(), list)}
        log.close()
        out = filepath.replace(".h5", ".png")
        plot_foc_results(data, out)
        print(f"Plot saved: {out}")

    elif args[0] in ("-h", "--help"):
        print(__doc__)

    else:
        print(f"Unknown option: {args[0]}")
        print(__doc__)

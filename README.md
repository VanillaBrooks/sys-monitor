# sys-monitor
Windows process monitor for checking need for low-latency conditions

## Project Purpose

This package serves to send get requests to a local server to stop high-bandwith traffic. The requests will be sent when a windows system process is running that is marked as requiring low latency. This is often in the form of competitive online games.

This package is intentionally hard-coded to my specific use-case to prevent local tampering. This leads to the absence of config files for IP addresses, process names to monitor, etc.

## Startup

The package can be compiled and run with `cargo`:

`cargo r --release`

Alternatively, windows x64 releases are available [here](https://github.com/VanillaBrooks/sys-monitor/releases)

## Usage

`sys-monitor.exe` can function correctly when run in an isolated `cmd` window. 

Alternatively, the process can be run permenantly with the command:

`sc.exe create sys-monitor binPath= "C:\path\to\executable.exe"`

Note that both the space after `binPath=` is important, as well as the quotes surrounding the path
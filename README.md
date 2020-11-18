# runtasktic

[![Rust](https://github.com/Joxit/runtasktic/workflows/Rust/badge.svg)](https://github.com/Joxit/runtasktic/actions?query=workflow%3ARust)
[![Crates.io version shield](https://img.shields.io/crates/v/runtasktic.svg)](https://crates.io/crates/runtasktic)
[![Crates.io license shield](https://img.shields.io/crates/l/runtasktic.svg)](https://crates.io/crates/runtasktic)

Runtasktic is a *fantastic* command-line task management tool for execution of regular long sequential or parallel tasks.

There are often tasks that we repeat several times in predefined orders. Some of these tasks can take time and we would like to be notified when it ends. This is why this project exists.

Describe your tasks in a YAML file, execute all of them with runtasktic in foreground or background. Configure the notification system, and be notified at the end of each task or when they are all finished.

## When should I need runtasktic ?

- when you have a redundant long running task list
- when you need a notification after a long task completion

## CLI

```
runtasktic 0.2.0
Jones Magloire @Joxit
Command-line task management tool for execution of regular long sequential or parallel tasks.

USAGE:
    runtasktic <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    dot     Export the configuration to a graph (needs graphviz/dot)
    exec    Execute a single command with notification
    help    Prints this message or the help of the given subcommand(s)
    run     Run tasks
```

### Run: all tasks of a configuration file

```
runtasktic-run 0.2.0
Run tasks

USAGE:
    runtasktic run [FLAGS] <config>

FLAGS:
    -b, --background    Run the task in background
    -h, --help          Prints help information
    -V, --version       Prints version information

ARGS:
    <config>    Configuration path (YAML)
```

### Exec: Simple command, just like nohup with notification

```
runtasktic-exec 0.2.0
Execute a single command with notification

USAGE:
    runtasktic exec [FLAGS] [OPTIONS] [command]...

FLAGS:
    -b, --background    Exec the command in background
    -h, --help          Prints help information
    -V, --version       Prints version information

OPTIONS:
    -c, --config <config>    Configuration path (YAML)

ARGS:
    <command>...    Command to execute
```

### Dot: Create a graph using graphviz of your configuration file

```
runtasktic-dot 0.2.0
Export the configuration to a graph (needs graphviz/dot)

USAGE:
    runtasktic dot <config> <image>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

ARGS:
    <config>    Path of the configuration file to visualize
    <image>     Path for the image. `dot` command is required
```

## Configuration examples

[Simple sample](https://github.com/Joxit/task-scheduler/blob/master/tests/resources/sample.yml)

[Play with concurrency](https://github.com/Joxit/task-scheduler/blob/master/tests/resources/concurrency.yml)

[Play with notification](https://github.com/Joxit/task-scheduler/blob/master/tests/resources/notification.yml)
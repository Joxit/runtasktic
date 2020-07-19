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

## Configuration examples

[Simple sample](https://github.com/Joxit/task-scheduler/blob/master/examples/resources/sample.yml)
[Play with concurrency](https://github.com/Joxit/task-scheduler/blob/master/examples/resources/concurrency.yml)
[Play with notification](https://github.com/Joxit/task-scheduler/blob/master/examples/resources/notification.yml)
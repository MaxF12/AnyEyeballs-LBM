# AnyEyeballs Orchestrator 

This repository (along with https://github.com/MaxF12/anyeyeballs) is the source code required for running the AnyEyeballs test application.
Its goal is to utilize the widely adopted HappyEyeballs protocol for client side load balancing. 

## Requirements
Rust version 1.51.0 or later is required for running the orchestrator.
## Setup
Before running the client, the address the orchestrator will listen on has to be configured in line 8 of main.rs. 
## Running
To run the orchestrator, simply execute cargo run from the AnyEyeballs_Orchestrator directory. 
## Configurable values
There are four parameters that can be configured for the orchestrator:

- MAX_NODES: The maximum number of clients allowed to connect to the orchestrator.
- LOAD_THRESHOLD: The percentage after which the orchestrator will start to turn of interfaces/clients.
- LOG_FILE: The path to the file where the log will be stored.
- LOG_INTERVAL: The interval in seconds in which the current state will be logged. 

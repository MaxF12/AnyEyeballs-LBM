# AnyEyeballs Load Balance Manager 

This repository (along with https://github.com/MaxF12/anyeyeballs) is the source code required for running the AnyEyeballs test implementation.
Its goal is to utilize the widely adopted Happy Eyeballs system to give implicit load balancing control to servers.

## Requirements
Rust version 1.51.0 as well as toml version 0.5.8 or later is required for running the orchestrator.
## Setup
Before running the LBM, the configuration in config.toml has to be updated. The configurable values are:

| Name                | Description                                                                                                                                                                        |
|---------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| lbm                 |                                                                                                                                                                                    |
| ip                  | The IP address the LBM will listen on.                                                                                                                                             |
| port                | The port the LBM will listen on.                                                                                                                                                   |
| max\_nodes          | The maximum number of nodes allowed in the network.                                                                                                                                |
| log                 |                                                                                                                                                                                    |
| file                | The path to the log file.                                                                                                                                                          |
| interval            | The time interval in seconds between each log entry.                                                                                                                               |
| balancer            |                                                                                                                                                                                    |
| load\_threshold     | The load threshold that if crossed will trigger the LBM to take action.                                                                                                            |
| relative\_threshold | The relative load share between IPv4 and IPv6 that if crossed will only trigger the shutdown of one socket, the one with the higher load. Otherwise, both sockets will be shut off. |
| mode                | The mode which the LBM will use to load balance. Can be 0 for the threshold or 1 for the round-robin style mode.                                                                   |

The LBM offers two different load balance modes as mentioned in the table above. The threshold based mode will lead to nodes being turned off once they reach the configured threshold, as long as there is at least one other node active.
The round-robin based mode will always turn off the node with the highest current load, as soon as its load reaches the preconfigured threshold. 
## Running
To run the orchestrator, simply execute "cargo run config.toml" from the AnyEyeballs_Orchestrator directory. 

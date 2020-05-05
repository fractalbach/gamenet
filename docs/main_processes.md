## System structure

### Processes

This document describes the processes and services in use at a 
high level.

#### Client
 * Handles user input.
 * Instantiates client events.
 * Receives events from the instance server.
 * Applies events (from self and server).
     * Audits selected events
 * Sends instantiated events to server.

#### Location shard
 * Maintains instance state for a defined region / time / group.
 * Receives events from clients
 * Generates instance events (NPCs, environment, etc)
 * Applies events (from instance server and clients)
    * Audits selected events.
 * Sends events to clients.

#### Global chat
 * Handles non-location specific chat messages.
 * Requires no knowledge of locations, instances, etc.

#### Entity server shard(s)
 * Manages a shard (horizontal partition) of entities data.
 * Monitors entity movement and notifies location shards when entity
    enters location shards' managed region.

#### Location manager
 * Tracks active location shards
 * Responds to requests for a shard server.
    * Gives location info for an existing location shard server if
            one exists that is appropriate.
    * Spawns and/or assigns a shard server 

#### Static data server
 * Stores data that is not updated during normal use. 
 * Responds to request for static data.
    * Number of entity servers
    * Locations of other servers.
    * Static assets


------------------------------------------------------


### Packages / crates

#### client

 * Client side web content / world visualization.
 * Implemented in Kotlin
 
#### open_client_core

 * Crate for client-side action handling, etc.
 * Provides the interface used by the client UI.
 * Handles events from the UI and server.
 * Implemented in Rust.
 
#### open_client

 * Handles game-specific client logic.

#### open_world_core

 * Crate for managing persistent world state.
 * Contains specs world instance.
 * Provides event traits
 * Game agnostic
 * Implemented in Rust.
 
#### open_world

 * Used by both clients and location servers.
 * Provides event specializations.
    * Provides audit implementations.
 * Implemented in Rust
 
#### procede

 * Procedural world generation
 * Produces:
    * terrain
    * building info 
    * prop placements
    * etc.
 
#### open_location_shard

 * Handles location instance server logic.
 
#### open_location_manager

 * Handles sharding of locations into location servers.
 * Provides RPC interface.

#### open_entity_shard

 * Handles entity tracking.
 * Provides entity specific data to location shards.
 * Implemented in Rust or Python.

#### chat

 * Handles global chat communications.
 * Implemented in Go or Python.

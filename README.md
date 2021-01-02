# derico

Generic state debouncer for complete graphs. This lib also includes a pin debouncer for embedded programming.

The name of this lib is derived from replacing bounce in debounce with ricochet and shorten it.
Don't look down on me. I'm not very creative.

## Description

The generic debouncer takes as generic arguments a state enum, e.g. `High` and `Low`, and a number type, i.e. must have a one and can be added (for the pin debouncer this simplt is u8).

The debouncer will emit an edge, when a debounced transition happens.
When polling the state, the debouncer will change internal state, if the same state was returned more often than a certain threshold consecutively.
A state transition will not be emitted, if the "transistion" is from a state to the same state.
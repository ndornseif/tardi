# TARDI VM
Tiny stack VM where every sequence of bytes is a valid program.
Named after the tiny animal that survives everything thrown at it.
## Architecture

## Bytecode
Instructions are encoded as bytes. During decoding they are taken modulo the total number of instructions. This means that every possible byte maps to some valid instruction.  
Note that this means adding new instructions will break everything that relies on this behaviour.
The VM is meant to allow genetic algorithms to explore a solution space more efficiently. A finalized program should then be 
adapted to some other execution platform or converted into the `canonical` version that does not rely on many of TARDIs more eccentric behaviours.

## The Stack
The stack size is set by the `MAX_STACK` constant, the data type it contains by the `MachineWord` type.

## Type agnostic behaviours

## ~~Quirks~~ Notable properties

## Instructions

## The disassembler

## License
This crate may be licensed under the [GNU Lesser General Public License, version 2.1](https://www.gnu.org/licenses/old-licenses/lgpl-2.1.html#SEC1).

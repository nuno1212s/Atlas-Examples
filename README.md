# Atlas Examples

This repository contains examples of how to use the [Atlas](https://github.com/nuno1212s/Atlas) framework for several applications.

## Examples

### Calculator
Calculator is a simple app built on Monolithic state architecture.

## Multithreading

All applications which have a ```Sync``` state are automatically multithreaded even if they are not explicitly marked as such and don't use our ScalableApp class and CRUD state.
This means that ordered operations are executed in the serialized order 
# Notes

This document contains development notes and project-wide todos.

**TODO** - consider reworking some of the constructors to take wrapper builders instead of unsafe variant enums.

**TODO** - Recording commands to a command buffer needs to synchronize over the command POOL as well.

**TODO** - Non dispatchable handle don't have to be unique. Does this mean wrappers shouldn't implement Eq, Hash and Ord?

A library for reading and writing DBPF encoded binary packages.

The main goals of this library are, in order of relative importance:
- compatibility
  DBPF is a fixed file format that will never receive any updates,
  so this library should be able to read/inspect ALL DBPF packages ever created,
  past prsent and future
- performance
  Packages should be able to be written by using only a single backwards seek by writing all entry contents, then the header and index.
  This should also make reading the packages faster as the header and index are now iin a single contiguous block.
- losslessness
  All file contents should be written without any observable end-user effects; this means that changes to internal 
  layouts and representations are allowed (for convenience or perfomance reasons)
  as long as the file decodes to the exact same contents.

Non-goals:
- partial updates
  Older versions of DBPF have the ability to update, delete or insert objects by editing only part of the file on-disk.
  Since this behavior is not portable across DBPF versions and it complicates file handling, this will never be implemented.
- writing file holes
  same as above, this needlessly complicates writing for a non-feature. The hole index can however be decoded by this library if so desired.

Other great DBPF resouces and libraries:
- TBA 
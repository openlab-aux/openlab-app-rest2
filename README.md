# openlab-app-rest2

Stupid silly REST server for the OpenLab app API

By default all the values are zeroized upon dropping on a best-effort basis.
Also all the memory is locked to ephemeral RAM and won't be swapped to disk as per the `mlockall` syscall.

Has a full OpenAPI documentation available as well. So yeah.

## Options

### `--no-mlock`

Doesn't lock the entire virtual address space in RAM, preventing swapping.

**VERY MUCH DISCOURAGED**. This server stores presence data in-memory.
In case of a raid or theft of the disk, adversaries might be able to restore presence data from swapped memory.

## License

This software is licensed under the MPL-2.0. All your intentional contributions will be licensed under the same terms.

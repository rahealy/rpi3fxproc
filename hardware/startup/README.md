# Startup

Crates to start up and initialize the Raspberry Pi hardware.

Sets up exceptions, MMU, core and priviledges. 

CPU is put into EL1, core 0 before calling the main() function as declared below:

```
//Minimal main() function.

#[export_name = "main"]
fn main() -> ! { loop{} }
```

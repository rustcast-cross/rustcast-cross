# Modes

> [!WARNING]
>
> This hasn't been implemented in the fork yet, this is just kept for when i get around to porting it over

Modes are similar to macos focus modes.

You can run a shell script to disable / enable your WM, turn on / off your notifications, or do pretty much anything.

Modes are still a relatively trial feature, so don't expect a lot from it just yet (It is v0.5.9 at the time of writing this)
They lack a bunch of features, but they will be improved over time. 

Modes are fairly simple to add, but much more annoying to make.
The different between modes and shell scripts is that modes are meant to be shown in the menu bar icon and in the rustcast footer. 

```toml
[modes]
default = "~/.config/rustcast/hello_modes.sh" # overrides the "do nothing" default mode
presentation = "~/.config/rustcast/presentation.sh"
```

This creates 1 new mode and overrides the default mode.

Now, when u switch to the presentation mode, the presentation.sh file (which should be stored inside the rustcast config directory) is run. 
And when you switch back to the default mode, it will run the "hello_modes.sh" (in the place we defined) 



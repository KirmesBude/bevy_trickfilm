v0.11.0
================================================================================================================================
Update to bevy 0.16

v0.10.0
================================================================================================================================
Add support for generic Time (e.g. Virtual and Real)

v0.9.1
================================================================================================================================
Implement Clone for some public structs

v0.9.0
================================================================================================================================
Support for asset based, frame based animation events :)
- New animation event example
- New animation event derive macro for entity targeted events
Rework entire asset loading to allow for reflected animation events
Add very simple frame based event trigger detection
Some new interfaces on AnimationPlayer2D
Use bevy provided Animation SystemSet instead of AnimationPlayer2DSystemSet

v0.8.0
================================================================================================================================
Support and recommend .trickfilm.ron file extension, .trickfilm still works, but is no longer recommended.
Reworked public facing API and internal behaviour.
Quick migration guide:
is_playing_clip -> clip_playing
is_finished -> finished
set_repeat -> set_repeat_mode
is_playback_reversed -> reversed
is_paused -> paused

v0.7.0
================================================================================================================================
Update to bevy v0.14.

v0.6.0
================================================================================================================================
Update to bevy v0.13.

v0.5.0
================================================================================================================================
Removed support for collection of sprites. Only texture atlas are supported now. Refer to bevy examples on how to create a 
texture atlas from sprites or use bevy_titan.
Change the contents of the trickfilm manifest file. Now the animation itself no longer references the texture atlas. This
allows reusing the same animation for multiple texture atlases with the same layout. 
Add example for usage with bevy_asset_loader.

v0.4.0
================================================================================================================================
Update to bevy v0.12.

v0.3.0
================================================================================================================================
Update to bevy v0.11.

v0.2.0
================================================================================================================================
Update to bevy v0.10.
Add support for sprite animations.
Model AnimationPlayer2D closer to bevy AnimationPlayer. 

v0.1.0
================================================================================================================================
Initial release.

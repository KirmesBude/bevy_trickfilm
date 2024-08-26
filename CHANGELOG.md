v0.7.0
================================================================================================================================
Reworked public facing API and internal behaviour.

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

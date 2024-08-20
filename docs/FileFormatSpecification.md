# Trickfilm RON file format specification.

## Trickfilm
| Type                           | Necessity | Description |
|--------------------------------|-----------|-------------|
| Map of String,[TrickfilmEntry] | mandatory | All named animation clips of this animation clip set. Can not be empty. If multiple animation clips with the same name are defined, only the last entry is considered. |

## TrickfilmEntry
| Field               | Type                      | Necessity | Description |
|---------------------|---------------------------|-----------|-------------|
| keyframes           | [TrickfilmEntryKeyframes] | mandatory | Keyframes of this animation clip corresponding to the indices in the texture atlas. |
| keyframe_timestamps | Option of Vector of f32   | optional  | Timestamp of the corresponding keyframe of this animation clip in seconds. Default value is None, but will be calculated so all keyframes are equally distributed along the entire duration. |
| duration            | f32                       | mandatory | Duration of this animation clip in seconds. Must be greater than the maximum keyframe timestamp. |

## TrickfilmEntryKeyframes
| Variant        | Description |
|----------------|-------------|
| KeyframesVec   | Vec of usize corresponding to individual keyframes. |
| KeyframesRange | Range of ussize corresponding to a range of keyframes. |

[TrickfilmEntry]: #trickfilmentry
[TrickfilmEntryKeyframes]: #trickfilmentrykeyframes

# Ship web viewer

Shipyard is a kind of chaos. Web viewer is attempt to direct all from one point. No install, no files, no versions just click the link and see.

![Untitled video - Made with Clipchamp](https://github.com/skokovin/putout/assets/13080037/514d2e2b-2522-4032-9699-3133bf80d14b)


## Intro

When building a ship or something large, there’s typically a 3D model with attributes. Usually, these models can take up a lot of gigabytes. Is there a way to transfer the model over the network and display it in a browser? As far as I know, Chrome can address up to 16GB per tab, also need a way to package a mesh data. This model of a 120 meter vessel takes up about 6GB of memory, packed into 500 MB.

## Basic Tools

Rust - philosophy to do it right. </br>
WGPU - The brilliant cross-platform, safe, pure-Rust graphics API.</br>
WINIT - for window handling.</br>
wasm-pack - to move all to WebAssembly.</br>
Angular - to stay on static typing (other project)

## Demos

Video instruction [YouTube](https://www.youtube.com/watch?v=E0fKqEAThts). </br>
[200mb ship](https://viewer004-8db15.web.app/). </br>
[500mb ship](https://viewer701-f462d.web.app/). </br>


## Features

- Hide/Selct
- 6 sliders
- Measuring
- ID for DB requests
- Transparency
- Reset center of rotation
- Orbit/FPS cameras
- Primitive Snap


## Acknowledgements

 - [NAUTIC. A lot of perfect ships there ](https://www.nautic.is/)

## Comments

At the first for interactivity i used AABB and ray tracing, but found better technic for this purpose - render to buffer. After each camera transformation special pipeline takes a snapshot, then algorithm analyzed area under mose cursor. Special pipeline use color and alfa chanels for pass back coords and id. </br>

Some logics of hide/select moved to shaders. </br>

## Disclaimer

It is experemental and not finished yet.

## License

All Reports in this Repository are licensed by Contributors
under the
[W3C Software and Document License](http://www.w3.org/Consortium/Legal/2015/copyright-software-and-document).

Contributions to Specifications are made under the
[W3C CLA](https://www.w3.org/community/about/agreements/cla/).

Contributions to Test Suites are made under the
[W3C 3-clause BSD License](https://www.w3.org/Consortium/Legal/2008/03-bsd-license.html)

Contributions to Software are made under the
[GPU for the Web 3-Clause BSD License](https://github.com/gpuweb/admin/blob/master/SourceCodeLicense/LICENSE.txt)


Run `ng e2e` to execute the end-to-end tests via a platform of your choice. To use this command, you need to first add a package that implements end-to-end testing capabilities.

## Further help

s.kokovin@gmail.com

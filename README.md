Still a work in progress.

Generates the shapes and then serializes them to a JSON file with a list of 
vertices (`positions`) and a list of triangle faces (`cells`) that index into 
the list of vertices. Suitable for input into [Three.js's 
BufferGeometry](https://threejs.org/docs/#api/en/core/BufferGeometry) or 
[regl](https://github.com/regl-project/regl/blob/gh-pages/example/camera.js).

Icosahedrons can be generated significantly faster than Three.js's version in 
JavaScript (which I pretty much copied into Rust).

Trunacated icosahedrons (I call them hexspheres) are a bit slower to generate 
since they are made by generating a icosahedron and then subdividing it into 
hexagon and pentagon faces. I still have some work to do to improve that code.

I'm still having issues rendering hexspheres of detail level 5 and higher and 
icosahedrons of detail level of 7 and higher, so I'm not sure if those are 
accurate.

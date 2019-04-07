Generates the shapes and then serializes them to a JSON file with a list of 
vertices (`positions`) and a list of triangle faces (`cells`) that index into 
the list of vertices. Suitable for input into [Three.js's 
BufferGeometry](https://threejs.org/docs/#api/en/core/BufferGeometry) or 
[regl](https://github.com/regl-project/regl/blob/gh-pages/example/camera.js).

Icosahedrons can be generated significantly faster than Three.js's version in 
JavaScript (which I pretty much copied into Rust).

Trunacated icosahedrons (I call them hexspheres) are a bit slower to generate 
since they are made by generating a icosahedron and then subdividing it into 
hexagon and pentagon faces.

When rendering hexspheres of detail level 5 and higher and icosahedrons of 
detail level of 7 and higher in WebGL, make sure to enable the 
[`OES_element_index_uint`](https://developer.mozilla.org/en-US/docs/Web/API/OES_element_index_uint) 
extension since the number of vertices might overflow the normal int index.

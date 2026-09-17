# Viewer Architecture Review Gate

## Conclusion

The compiled viewer idea is complete only when it is defined as a general compiled-model graph plus an authoritative render artifact, not as a fixed CAD tree or a dumb GLB viewer.

### Required boundaries

```text
.NET Framework = semantic build authority
Rust kernel    = engineering/geometry/topology authority
Manifest      = compiled semantic/binding contract
GLB           = runtime visualization artifact
React         = rendering/navigation/inspection client
```

### Required model properties

- arbitrary recursive hierarchy (assembly may contain assemblies to any depth);
- extensible node kinds;
- extensible typed metadata;
- explicit non-tree relationships;
- stable semantic IDs independent of array order/display names;
- object-to-render bindings;
- authoritative face/topology bindings;
- source/provenance bindings;
- visibility/selection/focus capabilities;
- build identity linking manifest and GLB;
- explicit versioning and structured diagnostics.

### Critical gap

The current kernel specification is still a native 2D surface. Production 3D GLB export and the corresponding topology binding contract must therefore be implemented before the browser viewer can be called fully integrated.

### V1 viewer

React + TypeScript + Three.js/React Three Fiber. Read-only compiled-result navigation: recursive model tree, metadata inspector, object/face selection, show/hide, fit, focus, orbit, pan and zoom. No CAD solving or source mutation.

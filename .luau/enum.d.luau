declare Enum: {
    EasingStyle: { Linear: number, Sine: number, Quad: number },
    EasingDirection: { In: number, Out: number, InOut: number },
    PartType: { Block: number, Ball: number, Cylinder: number },
    CollisionFidelity: { Default: number, Hull: number, Box: number, PreciseConvexDecomposition: number },
    ColliderRenderMode: { ColliderShapes: number, RigidbodyAxes: number, MultiBodyJoints: number, ImpulseJoints: number, Joints: number, SolverContacts: number, Contacts: number, ColliderAABBS: number },
    Material: { Plastic: "Plastic", Neon: "Neon" },
}

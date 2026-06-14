export type Connection = {
    Disconnect: (self: Connection) -> ()
}

export type Signal<T...> = {
    Connect: (self: Signal<T...>, callback: (T...) -> ()) -> Connection
}

export type Vector2 = {
    X: number,
    Y: number,
} & {
    __add: (Vector2, Vector2) -> Vector2,
    __sub: (Vector2, Vector2) -> Vector2,
    __mul: (Vector2, number) -> Vector2,
}

export type Vector3 = {
    X: number,
    Y: number,
    Z: number,
} & {
    __add: (Vector3, Vector3) -> Vector3,
    __sub: (Vector3, Vector3) -> Vector3,
    __mul: (Vector3, number) -> Vector3,
}

export type Color3 = {
    R: number,
    G: number,
    B: number,
    Lerp: (self: Color3, goal: Color3, alpha: number) -> Color3,
}

export type CFrame = {
    Position: Vector3,
    LookVector: Vector3,
    RightVector: Vector3,
    UpVector: Vector3,
    X: number,
    Y: number,
    Z: number,
    Inverse: (self: CFrame) -> CFrame,
    Lerp: (self: CFrame, goal: CFrame, alpha: number) -> CFrame,
    ToEulerAnglesYXZ: (self: CFrame) -> (number, number, number),
} & {
    __mul: (CFrame, CFrame) -> CFrame,
}

export type UDim2 = {}

export type Instance = {
    Name: string,
    Parent: Instance?,
    Clone: (self: Instance) -> Instance,
    Destroy: (self: Instance) -> (),
}

export type Part = Instance & {
    Position: Vector3,
    CFrame: CFrame,
    Size: Vector3,
    Color: Color3,
    Transparency: number,
    Material: string,
    Touched: Signal<Instance>,
}

export type Camera = Instance & {
    FirstPerson: boolean,
    Distance: number,
    Sensitivity: number,
    Fov: number,
    CFrame: CFrame,
    SetSubject: (self: Camera, subject: Instance) -> (),
    ClearSubject: (self: Camera) -> (),
}

export type Collider = Instance & {
    Size: Vector3,
}

export type Rigidbody = Instance & {}

export type ScreenGui = Instance & {}

export type Frame = Instance & {
    Size: UDim2,
    Position: UDim2,
    AnchorPoint: Vector2,
    Transparency: number,
    BackgroundColor3: Color3,
}

export type BloomEffect = Instance & {
    Intensity: number,
    Size: number,
    Threshold: number
}

export type Workspace = Instance

export type Lighting = Instance & {
    Ambient: Color3,
    Brightness: number,
    GlobalShadows: boolean,
    ClockTime: number,
    TimeOfDay: string,
    FogColor: Color3,
    FogEnd: number,
    FogStart: number,
}

export type RunService = {
    RenderStepped: Signal<number>,
}

export type UserInputService = {
    InputBegan: Signal<string, string>,
    InputEnded: Signal<string, string>,
}

declare Vector2: {
    new: (x: number, y: number) -> Vector2,
    zero: () -> Vector2,
    one: () -> Vector2,
}

declare Vector3: {
    new: (x: number, y: number, z: number) -> Vector3,
    zero: Vector3,
    one: Vector3,
}

declare Color3: {
    new: (r: number, g: number, b: number) -> Color3,
    fromRGB: (r: number, g: number, b: number) -> Color3,
}

declare CFrame: {
    new: (x: number, y: number, z: number) -> CFrame,
    lookAt: (at: Vector3, look: Vector3) -> CFrame,
    fromEulerAnglesYXZ: (x: number, y: number, z: number) -> CFrame,
    identity: CFrame,
}

declare UDim2: {
    new: (xScale: number, xOffset: number, yScale: number, yOffset: number) -> UDim2,
    fromScale: (x: number, y: number) -> UDim2,
    fromOffset: (x: number, y: number) -> UDim2,
}

declare task: {
    spawn: <T...>(f: (T...) -> (), T...) -> thread,
    defer: <T...>(f: (T...) -> (), T...) -> thread,
    wait: (time: number?) -> number,
}

declare Part: { new: () -> Part }
declare Collider: { new: () -> Collider }
declare Rigidbody: { new: () -> Rigidbody }
declare ScreenGui: { new: () -> ScreenGui }
declare Frame: { new: () -> Frame }
declare BloomEffect: { new: () -> BloomEffect }

declare workspace: Workspace
declare Lighting: Lighting
declare Camera: Camera
declare RunService: RunService
declare UserInputService: UserInputService

export type CollisionRender = {
    Enable: boolean,
    DebugMode: number,
}

export type Connection = {
    Disconnect: (self: Connection) -> ()
}

export type Signal<T...> = {
    Connect: (self: Signal<T...>, callback: (T...) -> ()) -> Connection
}

type Vector2Meta = {
    __add: (Vector2, Vector2) -> Vector2,
    __sub: (Vector2, Vector2) -> Vector2,
    __mul: (Vector2, number) -> Vector2,
}

type Vector3Meta = {
    __add: (Vector3, Vector3) -> Vector3,
    __sub: (Vector3, Vector3) -> Vector3,
    __mul: (Vector3, number) -> Vector3,
}

export type Vector2 = typeof(setmetatable(
    {} :: { X: number, Y: number },
    {} :: Vector2Meta
))

export type Vector3 = typeof(setmetatable(
    {} :: { X: number, Y: number, Z: number, Unit: Vector3, Magnitude: number },
    {} :: Vector3Meta
))

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

export type TweenInfo = {
    Time: number,
    EasingStyle: number,
    EasingDirection: number,
    RepeatCount: number,
    Reverses: boolean,
    DelayTime: number,
}

export type Tween = {
    Completed: Signal<>,
    Play: (self: Tween) -> (),
    Pause: (self: Tween) -> (),
    Cancel: (self: Tween) -> (),
}

export type Instance = {
    Name: string,
    Parent: Instance?,
    Clone: (self: Instance) -> Instance,
    Destroy: (self: Instance) -> (),
    GetChildren: (self: Instance) -> { Instance },
    GetDescendants: (self: Instance) -> { Instance },
    FindFirstChild: (self: Instance, name: string) -> Instance?,
    IsDescendantOf: (self: Instance, descendant: Instance) -> boolean,
}

export type BasePart = Instance & {
    Position: Vector3,
    CFrame: CFrame,
    Size: Vector3,
    Color: Color3,
    Transparency: number,
    Material: string,
    Touched: Signal<BasePart>,
}

export type Part = BasePart & {
    Shape: number
}

export type MeshPart = BasePart & {
    MeshId: string,
    CollisionFidelity: number
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
    Friction: number,
    CanCollide: boolean,
    Restitution: number,
}

export type Rigidbody = Instance & {
    Anchored: boolean,
    GravityScale: number,
}

export type CharacterController = Instance & {
    MoveDirection: Vector3,
    WalkSpeed: number,
    Jump: boolean,
    JumpPower: number,
}

export type ScreenGui = Instance & {}

export type GuiObject = Instance & {
    AnchorPoint: Vector2,
    Position: UDim2,
    Size: UDim2,
    BackgroundTransparency: number,
    BackgroundColor3: Color3,
    Visible: boolean,
}

export type Frame = GuiObject

export type TextLabel = GuiObject & {
    Text: string,
    TextColor3: Color3,
    TextSize: number
}

export type ImageLabel = GuiObject & {
    Image: string
}

export type TextButton = GuiObject & {
    Text: string,
    TextColor3: Color3,
    TextSize: number,
    MouseButton1Click: Signal<>,
    MouseEnter: Signal<>,
    MouseLeave: Signal<>,
}

export type ImageButton = GuiObject & {
    Image: string,
    MouseButton1Click: Signal<>,
    MouseEnter: Signal<>,
    MouseLeave: Signal<>,
}

export type BloomEffect = Instance & {
    Intensity: number,
    Size: number,
    Threshold: number
}

export type Sky = Instance & {
    CubemapPath: string,
}

export type Atmosphere = Instance & {
    Density: number,
    Color: Color3,
    Decay: Color3,
    Glare: number,
    Haze: number,
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

declare CollisionRender: CollisionRender

declare Vector2: {
    new: (x: number, y: number) -> Vector2,
    zero: Vector2,
    one: Vector2,
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

declare TweenInfo: {
    new: (time: number?, style: number?, direction: number?, repeat_count: number?, reverses: boolean?, delay: number?) -> TweenInfo,
}

declare TweenService: {
    Create: (self: any, instance: Instance, info: TweenInfo, properties: { [string]: any }) -> Tween,
}

declare task: {
    spawn: <T...>(f: (T...) -> (), T...) -> thread,
    defer: <T...>(f: (T...) -> (), T...) -> thread,
    wait: (time: number?) -> number,
}

declare Part: { new: () -> Part }
declare MeshPart: { new: () -> MeshPart }
declare Collider: { new: () -> Collider }
declare Rigidbody: { new: () -> Rigidbody }
declare CharacterController: { new: () -> CharacterController }
declare ScreenGui: { new: () -> ScreenGui }
declare Frame: { new: () -> Frame }
declare TextLabel: { new: () -> TextLabel }
declare ImageLabel: { new: () -> ImageLabel }
declare TextButton: { new: () -> TextButton }
declare ImageButton: { new: () -> ImageButton }
declare BloomEffect: { new: () -> BloomEffect }
declare Sky: { new: () -> Sky }
declare Atmosphere: { new: () -> Atmosphere }

declare workspace: Workspace
declare Lighting: Lighting
declare Camera: Camera
declare RunService: RunService
declare UserInputService: UserInputService

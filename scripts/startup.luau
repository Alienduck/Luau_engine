local function newPart(name: string, position: Vector3, size: Vector3): Part
	local part: Part = Part.new()
	part.Name = name
	part.Parent = workspace
	part.Size = size
	part.Position = position
	return part
end

local subject = newPart("Subject", Vector3.new(0, 1, 0), Vector3.one)
subject.Transparency = 0
subject.Material = "Neon"

local speed = 25
local keys_pressed = {}

UserInputService.InputBegan:Connect(function(key: string, _: string)
	keys_pressed[key] = true
	if key == "KeyQ" then
		Camera.FirstPerson = not Camera.FirstPerson
	end
end)

UserInputService.InputEnded:Connect(function(key: string, _: string)
	keys_pressed[key] = false
end)

RunService.RenderStepped:Connect(function(delta: number)
	local camLook = Camera.CFrame.LookVector
	local camRight = Camera.CFrame.RightVector

	camLook = Vector3.new(camLook.X, 0, camLook.Z).Unit
	camRight = Vector3.new(camRight.X, 0, camRight.Z).Unit

	local dir = Vector3.zero

	if keys_pressed["KeyW"] then
		dir = dir + camLook
	end
	if keys_pressed["KeyS"] then
		dir = dir - camLook
	end
	if keys_pressed["KeyA"] then
		dir = dir - camRight
	end
	if keys_pressed["KeyD"] then
		dir = dir + camRight
	end

	if keys_pressed["Space"] then
		dir = dir + Vector3.new(0, 1, 0)
	end
	if keys_pressed["ControlLeft"] then
		dir = dir + Vector3.new(0, -1, 0)
	end

	if dir.X ~= 0 or dir.Y ~= 0 or dir.Z ~= 0 then
		if dir.Magnitude > 0 then
			dir = dir.Unit
		end

		subject.Position = subject.Position + (dir * (speed * delta))
	end
end)

local function random_color(): Color3
	return Color3.new(math.random(0, 255) / 255, math.random(0, 255) / 255, math.random(0, 255) / 255)
end

local subject_goal = random_color()
RunService.RenderStepped:Connect(function(delta: number)
	subject.Color = subject.Color:Lerp(subject_goal, delta)
	if subject.Color == subject_goal then
		subject_goal = random_color()
	end
end)

Camera.Distance = 20
Camera:SetSubject(subject)

Lighting.Brightness = 10
Lighting.Ambient = Color3.fromRGB(0, 0, 100)
Lighting.GlobalShadows = true
Lighting.ClockTime = 0
Lighting.FogStart = 20
Lighting.FogEnd = 500
print(Lighting.TimeOfDay)

local CLOCK_MULTIPLIER = 2.4

RunService.RenderStepped:Connect(function(delta: number)
	Lighting.ClockTime += (delta / 3600) * CLOCK_MULTIPLIER
end)

local bloom = BloomEffect.new()
bloom.Intensity = 0
bloom.Size = 24.0
bloom.Threshold = 1
bloom.Parent = Lighting

local bloom_sens = true

RunService.RenderStepped:Connect(function(delta: number)
	if bloom_sens then
		bloom.Intensity += delta * 1.5
		if bloom.Intensity > 6 then
			bloom_sens = false
		end
	else
		bloom.Intensity -= delta * 1.5
		bloom_sens = bloom.Intensity < 0
	end
end)

local sky = Sky.new()
sky.CubemapPath = "skybox/sky_skybox.ktx2"
sky.Parent = Lighting

local atmosphere = Atmosphere.new()
atmosphere.Parent = Lighting
atmosphere.Density = 0

local atmo_start_tween = TweenService:Create(atmosphere, TweenInfo.new(5), { Density = 1 })
local atmo_end_tween = TweenService:Create(atmosphere, TweenInfo.new(5), { Density = 0 })

atmo_start_tween.Completed:Connect(function()
	atmo_end_tween:Play()
end)
atmo_end_tween.Completed:Connect(function()
	atmo_start_tween:Play()
end)
atmo_start_tween:Play()

local rain = require("scripts/rain")
rain.Start()
local container = require("scripts/test_ui")

UserInputService.InputBegan:Connect(function(k_input: string, m_input: string)
	if k_input == "KeyF" then
		container.Visible = not container.Visible
	end
end)

local mesh_part = MeshPart.new()
mesh_part.Name = "map"
mesh_part.Parent = workspace
mesh_part.CollisionFidelity = Enum.CollisionFidelity.PreciseConvexDecomposition
mesh_part.MeshId = "models/chicken_idk_map/scene.gltf#Scene0"
mesh_part.Position = Vector3.zero
local _cl = Collider.new()
_cl.Parent = mesh_part
local _rb = Rigidbody.new()
_rb.Anchored = true
_rb.Parent = mesh_part

Camera:SetSubject(mesh_part)

local count_down = 0
local time_to_wait = math.random(1, 6)

RunService.RenderStepped:Connect(function(delta: number)
	count_down += delta
	if count_down < time_to_wait then
		return
	end
	count_down = 0
	local workspace_desc = workspace:GetDescendants()
	local random_des = workspace_desc[math.random(1, #workspace_desc)]
	Camera:SetSubject(random_des)
	print(`Change subject for {random_des.Name}`)
end)

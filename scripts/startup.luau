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

print(typeof(subject))

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

Lighting.Brightness = 0.2
Lighting.Ambient = Color3.fromRGB(0, 0, 255)
Lighting.GlobalShadows = true
Lighting.ClockTime = 23
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
atmosphere.Density = 0.5
atmosphere.Haze = 2.8
atmosphere.Color = Color3.fromRGB(200, 100, 255)
atmosphere.Glare = 0.5
atmosphere.Decay = Color3.fromRGB(255, 90, 80)

local rain = require("scripts/rain")
rain.Start()
local container = require("scripts/test_ui")

UserInputService.InputBegan:Connect(function(k_input: string, m_input: string)
    if k_input == "KeyF" then
        container.Visible = not container.Visible
    end
end)

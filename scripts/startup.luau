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
    return Color3.new(math.random(0, 100) / 100, math.random(0, 100) / 100, math.random(0, 100) / 100)
end

local subject_goal = random_color()
RunService.RenderStepped:Connect(function(delta: number)
    subject.Color:Lerp(subject_goal, 0.1)
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

-- Lighting.FogStart = 5
-- Lighting.FogEnd = 20
-- Lighting.FogColor = Color3.fromRGB(20, 20, 150)

local bloom = BloomEffect.new()
bloom.Intensity = 1.0
bloom.Size = 24.0
bloom.Threshold = 1.5
bloom.Parent = Lighting

local rain = require("scripts/rain")
rain.Start()

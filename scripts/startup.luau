local function newPart(name: string, position: Vector3, size: Vector3): Part
    local part: Part = Part.new()
    part.Name = name
    part.Parent = workspace
    part.Size = size
    part.Position = position
    return part
end

local subject = newPart("Subject", Vector3.zero, Vector3.one)
subject.Transparency = 0
subject.Material = "Neon"

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

Lighting.FogStart = 5
Lighting.FogEnd = 20
Lighting.FogColor = Color3.fromRGB(20, 20, 150)

local bloom = BloomEffect.new()
bloom.Intensity = 5
bloom.Size = 12
bloom.Threshold = 1
bloom.Parent = Lighting

local rain = require("scripts/rain")
rain.Start()

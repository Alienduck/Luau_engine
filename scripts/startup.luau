local function newPart(name: string, position: Vector3, size: Vector3): Part
    local part: Part = Part.new()
    part.Name = name
    part.Parent = workspace
    part.Size = size
    part.Position = position
    return part
end

local subject = newPart("Subject", Vector3.zero, Vector3.one)
subject.Transparency = 1

Camera.Distance = 20
Camera:SetSubject(subject)

Lighting.Brightness = 0.2
Lighting.Ambient = Color3.fromRGB(0, 0, 255)
Lighting.GlobalShadows = true

local rain = require("scripts/rain")
rain.Start()

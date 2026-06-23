local environment = require("scripts/environment")
local player = require("scripts/player")
local ui = require("scripts/ui")

local map = environment.Start()
local subject = player.Start()
ui.Start()

Camera:SetSubject(map)
task.wait(1)
Camera:SetSubject(subject)

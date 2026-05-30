-- local p = Part.new()
-- p.Position = Vector3.new(0, 0, 0)
-- p.Color = Color3.new(1, 0.2, 0.2)

-- local p2 = Part.new()
-- p2.Position = Vector3.new(0, 3, 0)
-- p2.Size = Vector3.new(1, 2, 1)
-- p2.Color = Color3.new(0.5, 0.5, 1)

-- task.wait(1)
-- p2:Destroy()

-- local forward = true
-- while true do
--     task.wait(1/60)
--     if forward then
--         for i = 1, 100 do
--             task.wait(1/60)
--             p.Position = Vector3.new(0, 0, i * 0.05)
--             p.Color = Color3.new(1 - i / 100, 0.2, 0.2)
--         end
--     else
--         for i = 100, 1, -1 do
--             task.wait(1/60)
--             p.Position = Vector3.new(0, 0, i * 0.05)
--             p.Color = Color3.new(1 - i / 100, 0.2, 0.2)
--         end
--     end
--     forward = not forward
-- end

-- Plusieurs coroutines indépendantes qui bougent chacune un cube
for i = 1, 5 do
    local p = Part.new()
    p.Position = Vector3.new(i * 2, 0, 0)
    task.spawn(function()
        local t = 0
        while true do
            task.wait(1/60)
            t += 1/60
            p.Position = Vector3.new(i * 2, math.sin(t + i) * 2, 0)
        end
    end)
end

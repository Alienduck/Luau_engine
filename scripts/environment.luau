local env = {}

function env.Start()
	Lighting.Brightness = 10
	Lighting.Ambient = Color3.fromRGB(0, 0, 100)
	Lighting.GlobalShadows = true
	Lighting.ClockTime = 0
	Lighting.FogStart = 20
	Lighting.FogEnd = 500

	RunService.RenderStepped:Connect(function(delta: number)
		Lighting.ClockTime += (delta / 3600) * 2.4
	end)

	local sky = Sky.new()
	sky.CubemapPath = "skybox/sky_skybox.ktx2"
	sky.Parent = Lighting

	local bloom = BloomEffect.new()
	bloom.Intensity = 1
	bloom.Size = 1.0
	bloom.Threshold = 1
	bloom.Parent = Lighting

	local bloom_start = TweenService:Create(bloom, TweenInfo.new(3), { Intensity = 6 })
	local bloom_end = TweenService:Create(bloom, TweenInfo.new(3), { Intensity = -7 })
	bloom_start.Completed:Connect(function()
		bloom_end:Play()
	end)
	bloom_end.Completed:Connect(function()
		bloom_start:Play()
	end)
	bloom_start:Play()

	local atmosphere = Atmosphere.new()
	atmosphere.Parent = Lighting
	atmosphere.Density = 0

	local atmo_start = TweenService:Create(atmosphere, TweenInfo.new(5), { Density = 1 })
	local atmo_end = TweenService:Create(atmosphere, TweenInfo.new(5), { Density = 0 })
	atmo_start.Completed:Connect(function()
		atmo_end:Play()
	end)
	atmo_end.Completed:Connect(function()
		atmo_start:Play()
	end)
	atmo_start:Play()

	local store = require("./store")

	store.bloom:on_change(function(enabled: boolean)
		if enabled then
			bloom_start:Play()
		else
			bloom_start:Pause()
			bloom_end:Pause()
			bloom.Intensity = 0
		end
	end)

	store.atmo:on_change(function(enabled: boolean)
		if enabled then
			atmo_start:Play()
		else
			atmo_start:Pause()
			atmo_end:Pause()
			atmosphere.Density = 0
		end
	end)

	store.collision:on_change(function(enabled: boolean)
		CollisionRender.Enable = enabled
	end)

	UserInputService.InputBegan:Connect(function(key: string)
		if key == "KeyB" then
			store.bloom:set(not store.bloom:get())
		elseif key == "KeyV" then
			store.atmo:set(not store.atmo:get())
		elseif key == "KeyR" then
			store.collision:set(not store.collision:get())
		elseif key == "KeyZ" then
			CollisionRender.DebugMode = (CollisionRender.DebugMode + 1) % (Enum.ColliderRenderMode.ColliderAABBS + 1)
		end
	end)

	local rain = require("./rain")
	rain.Start()

	local model = Model.from(
		2 == 1 and "models/chicken_idk_map/scene.gltf#Scene0" or "models/stylized_classroom/scene.gltf#Scene0"
	)
	model.Name = "map"
	model.Parent = workspace
	model.CollisionFidelity = Enum.CollisionFidelity.PreciseConvexDecomposition

	model.Loaded:Connect(function()
		print("Le modèle 3D est chargé !")

		for _, part in ipairs(model:GetDescendants()) do
			if part.ClassName == "Part" then
				(part :: Part).Transparency = math.random(0, 100) / 100
			end
		end

		local cl = Collider.new()
		cl.Parent = model
		local rb = Rigidbody.new()
		rb.Anchored = true
		rb.Parent = model

		task.wait(2)
		model:Destroy()
	end)

	return model
end

return env

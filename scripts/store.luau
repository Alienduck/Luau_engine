local reactive = require("./reactive")

return {
	bloom = reactive.State(true),
	atmo = reactive.State(true),
	collision = reactive.State(false),
	uiVisible = reactive.State(true),
}

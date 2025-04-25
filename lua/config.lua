---@class Config
---@field db_path string [required] The path to the vimscape database
---@field batch_size integer The number of keys typed before processing the batch
local M = {
	db_path = "",
	batch_size = 50,
}

return M

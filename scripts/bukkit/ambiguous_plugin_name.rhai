let result = log.has_line("Ambiguous plugin name '{0}' for files 'plugins/{1}' and 'plugins/{2}' in 'plugins'", "AmbiguousPluginName");

if result != () {
  return result;
}


///

[AmbiguousPluginName]
header = "There are 2 files for plugin {0}"
solutions = ["Delete {1}", "Delete file {2}"]
level = "warn"
detail = "W001"

require 'octokit'
require 'tmpdir'
require 'fileutils'

REPO_OWNER = 'esrlabs'
REPO_NAME = 'chipmunk'

RAKE_COMMANDS = [
  'rake clean',
  'rake bindings:test:stream',
  # 'rake bindings:test:indexes',
  # 'rake bindings:test:search',
  'rake bindings:test:observe'
]

SHELL_SCRIPT_PATH = 'application/apps/rustcore/ts-bindings/spec'
SHELL_SCRIPT_NAME = 'setup_config.sh'

if ARGV.length > 1
  puts "Usage: ruby scripts/tools/run_benchmarks.rb <number_of_releases>"
  exit(1)
end

NUMBER_OF_RELEASES = ARGV[0].to_i == 0 ? 1 : ARGV[0].to_i

puts "running benchmarks for last #{NUMBER_OF_RELEASES} releases"

client = Octokit::Client.new()

# Fetch the latest releases from the GitHub repository
releases = client.releases("#{REPO_OWNER}/#{REPO_NAME}")

# Iterate over the specified number of releases
releases.take(NUMBER_OF_RELEASES).each_with_index do |release, index|
  puts "Processing release #{index + 1}: #{release.name}"

  ENV_VARS = {
    'JASMIN_TEST_CONFIGURATION' => './spec/benchmarks.json',
    'PERFORMANCE_RESULTS_FOLDER' => 'chipmunk_performance_results',
    'PERFORMANCE_RESULTS' => "Benchmark_#{release.tag_name}.json",
    # 'SH_HOME_DIR' => "/home/ubuntu",
    'SH_HOME_DIR' => "/Users/sameer.g.srivastava"
  }
  # Create a temporary directory for this release
  Dir.mktmpdir do |temp_dir|
    # Clone the repository into the temporary directory
    system("git clone --depth 1 --branch #{release.tag_name} https://github.com/#{REPO_OWNER}/#{REPO_NAME}.git #{temp_dir}")

    FileUtils.cp_r("#{SHELL_SCRIPT_PATH}/.", "#{temp_dir}/#{SHELL_SCRIPT_PATH}/.", verbose: true)

    # Change directory to the temporary directory
    Dir.chdir(temp_dir) do
      # Execute the shell script
      ENV_VARS.each do |key, value|
        ENV[key] = "#{value}"
      end

      system("printenv")

      if File.exist?("#{SHELL_SCRIPT_PATH}/#{ENV_VARS['JASMIN_TEST_CONFIGURATION'].gsub('./spec/', '')}")
        puts "File exists."
      else
        break
      end
      # Run each Rake command
      RAKE_COMMANDS.each do |command|
        system(command)
      end
    end
    system("cat #{ENV_VARS['SH_HOME_DIR']}/#{ENV_VARS['PERFORMANCE_RESULTS_FOLDER']}/Benchmark_#{release.tag_name}.json")
  end

  puts "Completed processing release #{index + 1}: #{release.name}"
end
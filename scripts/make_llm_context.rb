
def main()
    context = ""

    # list src/
    src_files = Dir.glob("src/**/*.rs").sort {|a, b|  a <=> b}.each do |src_file|
        puts "Reading #{src_file}"
        file = "```rust #{src_file}\n#{File.read(src_file)}\n```\n"
        context += file
    end
    
    puts context
end

main
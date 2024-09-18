require "redcarpet"

module CC
  module Analyzer
    module Formatters
      class HTMLFormatter < Formatter # rubocop: disable Metrics/ClassLength
        LANGUAGES = Hash.new { |_, ext| ext }.
                    merge(
                      # abap
                      # ada
                      "appraisals" => "ruby",
                      "as" => "actionscript",
                      "asm" => "nasm",
                      "bas" => "basic",
                      # c
                      "c++" => "cpp",
                      "capfile" => "ruby",
                      "cc" => "cpp",
                      "cfc" => "markup",
                      "cfm" => "markup",
                      "coffee" => "coffeescript",
                      "cp" => "cpp",
                      # cpp
                      "cr" => "crystal",
                      "cs" => "csharp",
                      "css" => %w[css css-extras],
                      "cu" => "cpp",
                      "cxx" => "cpp",
                      # d
                      # dart
                      # diff
                      "dockerfile" => "docker",
                      "dpr" => "pascal",
                      "erl" => "erlang",
                      "ex" => "elixir",
                      "f" => "fortran",
                      "f90" => "fortran",
                      "f95" => "fortran",
                      "feature" => "gherkin",
                      "for" => "fortran",
                      "fs" => "fsharp",
                      "fsi" => "fsharp",
                      "fsscript" => "fsharp",
                      "fsx" => "fsharp",
                      "gemfile" => "ruby",
                      "gemspec" => "ruby",
                      # glsl
                      # go
                      # groovy
                      "gvy" => "groovy",
                      "h" => "c",
                      "h++" => "cpp",
                      # haml
                      # handlebars
                      "hbr" => "handlebars",
                      "hh" => "cpp",
                      "hpp" => "cpp",
                      "hs" => "haskell",
                      "htm" => "markup",
                      "html" => "markup",
                      "hx" => "haxe",
                      "hxml" => "haxe",
                      "icn" => "icon",
                      "ijs" => "j",
                      # ini
                      "iol" => "jolie",
                      # java
                      "jl" => "julia",
                      "js" => "javascript",
                      # json
                      # jsx
                      "kt" => "kotlin",
                      "kts" => "kotlin",
                      # less
                      "lhs" => "haskell",
                      "lol" => "lolcode",
                      "lols" => "lolcode",
                      "ls" => "livescript",
                      # lua
                      "m" => "objective-c",
                      "mab" => "ruby",
                      # makefile
                      # markdown
                      "md" => "markdown",
                      # mel
                      "mkd" => "markdown",
                      "ml" => "ocaml",
                      "mli" => "ocaml",
                      "mm" => "objective-c",
                      # nim
                      # nix
                      "nsi" => "nsis",
                      "ol" => "jolie",
                      # oz
                      "pas" => "pascal",
                      "patch" => "diff",
                      "pde" => "processing",
                      "php" => %w[php php-extras],
                      "php3" => %w[php php-extras],
                      "php4" => %w[php php-extras],
                      "php5" => %w[php php-extras],
                      "phtml" => %w[php php-extras],
                      "pl" => "perl",
                      "pp" => "puppet",
                      "prawn" => "ruby",
                      "pro" => "prolog",
                      # properties
                      # pure
                      "py" => "python",
                      "py3" => "python",
                      "pyw" => "python",
                      "q" => "qore",
                      "qm" => "qore",
                      "qtest" => "qore",
                      # r
                      "rake" => "ruby",
                      "rakefile" => "ruby",
                      "rantfile" => "ruby",
                      "rb" => "ruby",
                      "rbw" => "ruby",
                      "rjs" => "ruby",
                      "rpdf" => "ruby",
                      "rs" => "rust",
                      "rst" => "rest",
                      "ru" => "ruby",
                      "rxml" => "ruby",
                      # sass
                      "sc" => "scala",
                      # scala
                      "scs" => "scheme",
                      # scss
                      "shader" => "glsl",
                      # sql
                      "ss" => "scheme",
                      "st" => "smalltalk",
                      "styl" => "stylus",
                      # swift
                      # tcl
                      "template" => "json",
                      "tex" => "latex",
                      # textile
                      "tmproj" => "markup",
                      "tpl" => "smarty",
                      "ts" => "typescript",
                      "v" => "verilog",
                      "vagrantfile" => "ruby",
                      "vhd" => "vhdl",
                      # vim
                      "xaml" => "markup",
                      "xhtml" => "markup",
                      "xml" => "markup",
                      # yaml
                      "yaws" => "erlang",
                      "yml" => "yaml",
                    ).freeze

        class Location
          CONTEXT_LINES = 2
          MAX_LINES = 10

          def initialize(source_buffer, location)
            @source_buffer = source_buffer
            @location = location
          end

          def begin_line
            @begin_line ||= line("begin")
          end

          def end_line
            @end_line ||= line("end")
          end

          def to_s
            [
              begin_line,
              end_line,
            ].uniq.join("-")
          end

          def start
            [begin_line - CONTEXT_LINES, 1].max
          end

          def line_offset
            start - 1
          end

          def code
            first_line = start
            last_line = [
              end_line + CONTEXT_LINES,
              begin_line + MAX_LINES + CONTEXT_LINES,
              source_buffer.line_count,
            ].min
            source_buffer.source.lines[(first_line - 1)..(last_line - 1)].join("")
          end

          private

          attr_reader :location, :source_buffer

          def line(type)
            if location["lines"]
              location["lines"][type]
            elsif location["positions"]
              position_to_line(location["positions"][type])
            end
          end

          def position_to_line(position)
            position["line"] || @source_buffer.decompose_position(position["offset"]).first
          end
        end

        class SourceFile
          def initialize(path, filesystem)
            @path = path
            @filesystem = filesystem
          end

          attr_reader :path

          def syntaxes
            ext = File.basename(path).split(".").last.downcase
            Array(LANGUAGES[ext])
          end

          def code
            filesystem.read_path(path)
          end

          def buffer
            @buffer ||= SourceBuffer.new(path, code)
          end

          def location(loc)
            Location.new(buffer, loc)
          end

          private

          attr_reader :filesystem
        end

        class Issue
          MARKDOWN_CONFIG = { autolink: true, fenced_code_blocks: true, no_intra_emphasis: true, tables: true }.freeze

          def initialize(data, filesystem)
            @data = data
            @filesystem = filesystem
          end

          def description
            data["description"]
          end

          def body
            @body ||=
              begin
                text = data.fetch("content", {}).fetch("body", "").strip
                unless text.empty?
                  markdown(text)
                end
              end
          end

          def source
            @source ||= SourceFile.new(
              data.fetch("location", {}).fetch("path", ""),
              filesystem,
            )
          end

          def location
            @location ||=
              Location.new(
                source.buffer,
                data["location"],
              )
          end

          def other_locations
            @other_locations ||=
              begin
                data.fetch("other_locations", []).map do |loc|
                  [SourceFile.new(loc["path"], filesystem), loc]
                end.to_h
              end
          end

          def categories
            data.fetch("categories", [])
          end

          def engine_name
            data["engine_name"]
          end

          def check_name
            data["check_name"].split(".").last
          end

          def severity
            data["severity"]
          end

          private

          attr_reader :data, :filesystem

          def markdown(text)
            html = Redcarpet::Render::HTML.new(
              escape_html: false,
              link_attributes: { target: "_blank" },
            )
            redcarpet = Redcarpet::Markdown.new(html, MARKDOWN_CONFIG)
            redcarpet.render(text)
          end
        end

        class IssueCollection
          SEVERITY_ORDER = {
            "info" => 0,
            "minor" => 1,
            "major" => 2,
            "critical" => 3,
            "blocker" => 4
          }.freeze

          def initialize(filesystem)
            @collection = []
            @filesystem = filesystem
          end

          def each(&block)
            collection.sort_by{ |issue| SEVERITY_ORDER.fetch(issue.severity, -1) }.reverse.each(&block)
          end

          def <<(issue)
            if issue.is_a? Hash
              issue = Issue.new(issue, filesystem)
            end
            collection.push(issue)
          end

          def any?
            collection.any?
          end

          def syntaxes
            collection.flat_map do |issue|
              issue.source.syntaxes
            end.uniq.sort
          end

          def categories
            collection.flat_map(&:categories).uniq.sort
          end

          def engines
            collection.map(&:engine_name).uniq.compact.sort
          end

          def severities
            collection.map(&:severity).uniq.sort_by{ |severity| SEVERITY_ORDER.fetch(severity, -1) }.reverse
          end

          private

          attr_reader :collection, :filesystem
        end

        class ReportTemplate
          include ERB::Util
          attr_reader :issues

          TEMPLATE_PATH = File.expand_path(File.join(File.dirname(__FILE__), "templates/html.erb"))

          def initialize(issues, filesystem)
            @issues = issues
            @filesystem = filesystem
          end

          def render
            template = File.read(TEMPLATE_PATH)
            ERB.new(template, trim_mode: "-").result(binding)
          end

          def project_name
            File.basename(filesystem.root)
          end

          def param(str)
            str.downcase.gsub(/\s+/, "-")
          end

          def params(values)
            values.map { |c| param c }.join(" ")
          end

          private

          attr_reader :filesystem
        end

        def finished
          puts ReportTemplate.new(issues, @filesystem).render
        end

        def failed(_)
          exit 1
        end

        private

        def issues
          @issues ||= IssueCollection.new(@filesystem)
        end

        def warnings
          @warnings ||= []
        end

        def measurements
          @measurements ||= []
        end
      end
    end
  end
end

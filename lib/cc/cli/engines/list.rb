# frozen_string_literal: true

module CC
  module CLI
    module Engines
      class List < EngineCommand
        SHORT_HELP = "List all available engines"

        def run
          say "Available engines:"
          engine_registry.
            sort_by { |engine, _| engine.name }.
            each do |engine, metadata|
              say "- #{engine.name}: #{metadata.description}"
            end
        end
      end
    end
  end
end

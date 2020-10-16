# frozen_string_literal: true

module CC
  module Analyzer
    class LocationDescription
      def initialize(source_buffer, location, suffix = "")
        @source_buffer = source_buffer
        @location = location
        @suffix = suffix
      end

      def to_s
        [
          lines,
          lines.blank? ? "" : suffix,
        ].join
      end

      private

      attr_reader :location, :suffix

      def lines
        @lines ||= begin
          if location["lines"]
            begin_line = location["lines"]["begin"]
            end_line = location["lines"]["end"]
          elsif location["positions"]
            begin_line = position_to_line(location["positions"]["begin"])
            end_line = position_to_line(location["positions"]["end"])
          end

          render_lines(begin_line, end_line)
        end
      end

      def render_lines(begin_line, end_line)
        if end_line == begin_line
          begin_line.to_s
        else
          "#{begin_line}-#{end_line}"
        end
      end

      def position_to_line(position)
        if position["line"]
          position["line"]
        else
          @source_buffer.decompose_position(position["offset"]).first
        end
      end
    end
  end
end

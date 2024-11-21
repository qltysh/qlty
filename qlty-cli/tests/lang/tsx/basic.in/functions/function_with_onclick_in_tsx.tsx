import React from "react";

interface ToggleOption {
  toggleType: string;
  externalKey?: string;
  disabled?: boolean;
  label: string;
}

interface Props {
  active: string;
  onChange: (key: string) => void;
}

function ToggleButtons(props: Props) {
  const effectiveToggleOpts: ToggleOption[] = []; // Mocked data

  return (
    <div className="data-toggle">
      {effectiveToggleOpts.map((toggleOption) => {
        const key = toggleOption.toggleType;
        const externalKey = toggleOption.externalKey || key;
        const disabled = toggleOption.disabled || false;

        return (
          <button
            key={key}
            className={`data-toggle__btn ${props.active === externalKey ? "active" : ""}`}
            type="button"
            disabled={disabled}
            onClick={() => props.onChange(externalKey)}
          >
            {toggleOption.label}
          </button>
        );
      })}
    </div>
  );
}

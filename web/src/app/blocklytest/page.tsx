'use client';
import * as Blockly from 'blockly/core';
import type React from 'react';
import { useEffect, useRef } from 'react';
import 'blockly/blocks';
import 'blockly/javascript';

const BlocklyTestPage: React.FC = () => {
  const blocklyDiv = useRef<HTMLDivElement>(null);
  const toolbox = {
    kind: 'flyoutToolbox',
    contents: [
      {
        kind: 'block',
        type: 'controls_if',
      },
      {
        kind: 'block',
        type: 'logic_compare',
      },
      {
        kind: 'block',
        type: 'math_number',
      },
      {
        kind: 'block',
        type: 'text',
      },
    ],
  };

  useEffect(() => {
    if (blocklyDiv.current) {
      Blockly.inject(blocklyDiv.current, {
        toolbox,
      });
    }
  }, []);

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100vh' }}>
      <h1>Blockly Demo</h1>
      <div
        ref={blocklyDiv}
        style={{ flex: 1, border: '1px solid #ccc', marginTop: '10px' }}
      />
    </div>
  );
};

export default BlocklyTestPage;

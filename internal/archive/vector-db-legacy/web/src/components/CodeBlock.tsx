import { Component, createSignal, Show, createMemo } from 'solid-js';

interface CodeBlockProps {
  code: string;
  language?: string;
  showLineNumbers?: boolean;
  copyable?: boolean;
}

const CodeBlock: Component<CodeBlockProps> = (props) => {
  const [copied, setCopied] = createSignal(false);
  let codeRef: HTMLDivElement;

  const handleCopy = async () => {
    const textToCopy = props.code.trim();
    try {
      await navigator.clipboard.writeText(textToCopy);
      
      // Visual feedback: select the text briefly
      if (codeRef) {
        const selection = window.getSelection();
        const range = document.createRange();
        range.selectNodeContents(codeRef);
        selection?.removeAllRanges();
        selection?.addRange(range);
        
        // Clear selection after showing feedback
        setTimeout(() => {
          selection?.removeAllRanges();
        }, 300);
      }
      
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.warn('Failed to copy code to clipboard:', err);
    }
  };

  const lines = createMemo(() => props.code.trim().split('\n'));
  const hasComments = createMemo(() => lines().some(line => line.trim().startsWith('#')));

  return (
    <div class="relative group">
      <div 
        class="bg-omen-gray-950 border border-omen-gray-800 text-omen-gray-100 rounded-lg font-mono text-sm overflow-x-auto cursor-pointer hover:border-omen-gray-700 transition-colors"
        onClick={props.copyable !== false ? handleCopy : undefined}
        title={props.copyable !== false ? 'Click to copy' : undefined}
      >
        <Show when={props.copyable !== false}>
          <button
            onClick={(e) => {
              e.stopPropagation();
              handleCopy();
            }}
            class="absolute top-2 right-2 px-3 py-1 text-xs bg-omen-gray-800 hover:bg-omen-indigo-500 text-omen-gray-300 hover:text-omen-white rounded-md transition-all opacity-0 group-hover:opacity-100 z-10"
            aria-label="Copy code"
          >
            <Show when={!copied()} fallback="Copied!">
              Copy
            </Show>
          </button>
        </Show>
        
        <div class="p-4" ref={codeRef!}>
          {lines().map((line, index) => {
            const isComment = line.trim().startsWith('#');
            const isEmpty = line.trim() === '';
            
            return (
              <div class="flex">
                <Show when={props.showLineNumbers}>
                  <span class="text-omen-gray-500 mr-4 select-none">{index + 1}</span>
                </Show>
                <span class={isComment ? 'text-omen-gray-500' : isEmpty ? '' : 'text-omen-gray-100'}>
                  {line || '\u00A0'}
                </span>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
};

export default CodeBlock;
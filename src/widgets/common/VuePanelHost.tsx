import { useEffect, useRef } from "react";
import { createApp, h, type Component } from "vue";

interface VuePanelHostProps {
  component: Component;
  className?: string;
}

export default function VuePanelHost({ component, className = "" }: VuePanelHostProps) {
  const mountRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    if (!mountRef.current) {
      return undefined;
    }

    // Mount the existing Vue panel inside the React shell so complex legacy pages
    // can keep their original structure and behavior during the migration phase.
    const app = createApp({
      render() {
        return h(component);
      },
    });
    app.mount(mountRef.current);

    return () => {
      app.unmount();
    };
  }, [component]);

  return <div ref={mountRef} className={className} />;
}

import type { ReactNode } from "react";
import { useState } from "react";
import PrimaryRail from "./PrimaryRail";
import SecondaryNav from "./SecondaryNav";
import type { RailItem, SecondaryNavItem } from "./types";

interface AppShellProps {
  railItems: RailItem[];
  activeRail: string;
  secondaryTitle: string;
  secondaryDescription?: string;
  secondaryItems: SecondaryNavItem[];
  activeSecondary: string;
  isSettingsActive?: boolean;
  children?: ReactNode;
  onSelectRail?: (key: string) => void;
  onSelectSecondary?: (key: string) => void;
  onOpenSettings?: () => void;
}

export default function AppShell({
  railItems,
  activeRail,
  secondaryTitle,
  secondaryDescription,
  secondaryItems,
  activeSecondary,
  isSettingsActive = false,
  children,
  onSelectRail,
  onSelectSecondary,
  onOpenSettings,
}: AppShellProps) {
  const [isSecondaryNavVisible, setSecondaryNavVisible] = useState(true);

  function handleRailSelect(key: string) {
    if (key === activeRail) {
      setSecondaryNavVisible((visible) => !visible);
      return;
    }
    setSecondaryNavVisible(true);
    onSelectRail?.(key);
  }

  return (
    <section className="page-shell">
      <aside className={`nav-stack ${isSecondaryNavVisible ? "" : "collapsed"}`}>
        <PrimaryRail
          items={railItems}
          activeKey={activeRail}
          isSecondaryNavVisible={isSecondaryNavVisible}
          isSettingsActive={isSettingsActive}
          onSelect={handleRailSelect}
          onToggleSecondaryNav={() => setSecondaryNavVisible((visible) => !visible)}
          onOpenSettings={onOpenSettings}
        />
        {isSecondaryNavVisible ? (
          <div className="secondary-nav-wrapper">
            <SecondaryNav
              title={secondaryTitle}
              description={secondaryDescription}
              items={secondaryItems}
              activeKey={activeSecondary}
              onSelect={onSelectSecondary}
            />
          </div>
        ) : null}
      </aside>
      <main className="content-wrap">{children}</main>
    </section>
  );
}

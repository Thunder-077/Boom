import type { LucideIcon } from "lucide-react";
import {
  Users,
  UserCheck,
  School,
  ClipboardList,
  ShieldCheck,
  Settings,
  Package,
  Shuffle,
  SlidersHorizontal,
  Palette,
  GitBranch,
  BarChart3,
  RefreshCw,
  FileText,
  FilePenLine,
  Calendar,
} from "lucide-react";

export const RAIL_ICONS: Record<string, LucideIcon> = {
  person: Users,
  badge: UserCheck,
  domain: School,
  school: ClipboardList,
  event_note: FileText,
  edit_file: FilePenLine,
  calendar_month: Calendar,
};

export const SECONDARY_NAV_ICONS: Record<string, LucideIcon> = {
  assignment: ClipboardList,
  badge: ShieldCheck,
  settings: Settings,
  inventory_2: Package,
  shuffle: Shuffle,
  tune: SlidersHorizontal,
  palette: Palette,
  calendar_month: Calendar,
  published_with_changes: GitBranch,
  query_stats: BarChart3,
  system_update: RefreshCw,
  edit_file: FilePenLine,
};

export interface RailItem {
  key: string;
  label: string;
  icon: keyof typeof RAIL_ICONS;
}

export interface SecondaryNavItem {
  key: string;
  label: string;
  icon?: keyof typeof SECONDARY_NAV_ICONS;
}

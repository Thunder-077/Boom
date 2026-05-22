import VuePanelHost from "../../../widgets/common/VuePanelHost";
import InvigilationPanelVue from "./InvigilationPanel.vue";

export default function MonitorConfigPanel() {
  return <VuePanelHost component={InvigilationPanelVue} className="vue-panel-host monitor-config-host" />;
}

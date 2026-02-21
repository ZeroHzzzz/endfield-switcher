<script setup lang="ts">
import { ref, onMounted, computed, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import SidebarPanel, { type AccountInfo } from './components/SidebarPanel.vue';
import MainPanel from './components/MainPanel.vue';
import DialogModal from './components/DialogModal.vue';

const accounts = ref<AccountInfo[]>([]);
const exePath = ref(localStorage.getItem('endfieldExePath') || 'C:\\Program Files\\Arknights Endfield\\Endfield.exe');
const currentFingerprint = ref('');
const selectedAccountId = ref<string | null>(null);
const newNote = ref('');
const loading = ref(false);
const statusMsgs = ref<{id: number, text: string}[]>([]);
let nextToastId = 0;

// Modal States
const modalState = ref({
  show: false,
  title: '',
  type: 'confirm' as 'confirm' | 'input',
  initialValue: '',
  confirmText: 'Confirm',
  danger: false,
  onConfirm: (val: string) => {},
});

const showModal = (
  title: string, 
  type: 'confirm'|'input', 
  onConfirm: (val: string) => void, 
  options: { initialValue?: string, confirmText?: string, danger?: boolean } = {}
) => {
  modalState.value = {
    show: true,
    title,
    type,
    onConfirm,
    initialValue: options.initialValue || '',
    confirmText: options.confirmText || 'Confirm',
    danger: options.danger || false,
  };
};

const showStatus = (msg: string) => {
  const id = nextToastId++;
  statusMsgs.value.push({ id, text: msg });
  setTimeout(() => {
    statusMsgs.value = statusMsgs.value.filter(t => t.id !== id);
  }, 3000);
};

const activeAccount = computed(() => {
  return accounts.value.find(a => a.fingerPrint === currentFingerprint.value);
});

const selectedAccount = computed(() => {
  return accounts.value.find(a => a.id === selectedAccountId.value) || activeAccount.value || accounts.value[0];
});

const loadData = async () => {
  try {
    const data = await invoke<AccountInfo[]>('load_accounts');
    accounts.value = data;
    currentFingerprint.value = await invoke<string>('get_current_fingerprint');

    // Auto select active account on first load
    if (!selectedAccountId.value && activeAccount.value) {
      selectedAccountId.value = activeAccount.value.id;
    } else if (!selectedAccountId.value && accounts.value.length > 0) {
      selectedAccountId.value = accounts.value[0].id;
    }
    
  } catch (e: any) {
    showStatus(e.toString());
  }
};

onMounted(() => {
  loadData();
});

watch(exePath, (newVal) => {
  localStorage.setItem('endfieldExePath', newVal);
});

const handleBackup = async () => {
  if (!newNote.value.trim()) {
    showStatus('Please enter a remark for the backup!');
    return;
  }
  loading.value = true;
  try {
    await invoke('backup_account', { note: newNote.value });
    newNote.value = '';
    showStatus('Backup successful!');
    await loadData();
  } catch (e: any) {
    showStatus('Error: ' + e.toString());
  }
  loading.value = false;
};

const handleSwitch = async (folderName: string) => {
  loading.value = true;
  try {
    await invoke('switch_account', { folderName });
    showStatus('Switched account successfully!');
    await loadData();
  } catch (e: any) {
    showStatus('Error: ' + e.toString());
  }
  loading.value = false;
};

const handleLaunch = async () => {
  try {
    await invoke('launch_game', { exePath: exePath.value });
    showStatus('Game launched!');
  } catch (e: any) {
    showStatus('Error: ' + e.toString());
  }
};

const handleSwitchAndLaunch = async (folderName: string) => {
  const accountToSwitch = accounts.value.find(a => a.folderName === folderName);
  if (accountToSwitch && accountToSwitch.fingerPrint !== currentFingerprint.value) {
    await handleSwitch(folderName);
  }
  await handleLaunch();
};

const handleSelect = (id: string) => {
  selectedAccountId.value = id;
};

const handleDelete = (id: string) => {
  showModal('Delete Backup Session?', 'confirm', async () => {
    loading.value = true;
    try {
      await invoke('delete_account', { id });
      showStatus('Deleted successfully!');
      await loadData();
    } catch (e: any) {
      showStatus('Error: ' + e.toString());
    }
    loading.value = false;
  }, { confirmText: 'Delete', danger: true });
};

const handleRename = (id: string, oldName: string) => {
  showModal('Edit Remark', 'input', async (newName: string) => {
    if (!newName || newName === oldName) return;
    loading.value = true;
    try {
      await invoke('rename_account', { id, newName });
      showStatus('Remark updated successfully!');
      await loadData();
    } catch (e: any) {
      showStatus('Error: ' + e.toString());
    }
    loading.value = false;
  }, { initialValue: oldName, confirmText: 'Save' });
};

const handleBrowse = async () => {
  const selected = await open({
    multiple: false,
    filters: [{ name: 'Executable', extensions: ['exe'] }]
  });
  if (selected && typeof selected === 'string') {
    exePath.value = selected;
  }
};
</script>

<template>
  <div class="layout">
    <SidebarPanel 
      :accounts="accounts"
      :currentFingerprint="currentFingerprint"
      :selectedAccountId="selectedAccountId"
      :loading="loading"
      @select="handleSelect"
      @rename="handleRename"
      @delete="handleDelete"
    />

    <div class="main-wrapper">
      <MainPanel 
        :selectedAccount="selectedAccount"
        :activeAccount="activeAccount"
        :loading="loading"
        v-model:exePath="exePath"
        v-model:newNote="newNote"
        @backup="handleBackup"
        @switchAndLaunch="handleSwitchAndLaunch"
        @launch="handleLaunch"
        @browse="handleBrowse"
      />
      
      <div class="toast-container">
        <TransitionGroup name="list">
          <div v-for="(msg, i) in statusMsgs" :key="msg.id" class="toast">
            {{ msg.text }}
          </div>
        </TransitionGroup>
      </div>
    </div>
    
    <DialogModal 
      v-model:show="modalState.show"
      :title="modalState.title"
      :type="modalState.type"
      :initial-value="modalState.initialValue"
      :confirm-text="modalState.confirmText"
      :danger="modalState.danger"
      @confirm="modalState.onConfirm"
    >
      <p v-if="modalState.type === 'confirm'">
        Are you sure you want to proceed? This action cannot be undone.
      </p>
    </DialogModal>
  </div>
</template>

<style scoped>
.layout {
  display: flex;
  height: 100vh;
  width: 100vw;
  background-color: var(--bg-base);
  color: var(--text-primary);
}
.main-wrapper {
  flex: 1;
  display: flex;
  flex-direction: column;
  position: relative;
}
.toast-container {
  position: absolute;
  bottom: 32px;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  flex-direction: column;
  gap: 8px;
  z-index: 50;
  pointer-events: none;
}
</style>

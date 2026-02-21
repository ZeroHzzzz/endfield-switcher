<script setup lang="ts">
import { ShieldCheck, Cross, Play, FileCog, ShieldAlert, Plus, FolderSearch, Settings2, RefreshCcw } from 'lucide-vue-next';
import type { AccountInfo } from './SidebarPanel.vue';

defineProps<{
  selectedAccount?: AccountInfo;
  activeAccount?: AccountInfo;
  exePath: string;
  loading: boolean;
  newNote: string;
}>();

const emit = defineEmits<{
  (e: 'update:newNote', value: string): void;
  (e: 'update:exePath', value: string): void;
  (e: 'backup'): void;
  (e: 'switchAndLaunch', folderName: string): void;
  (e: 'launch'): void;
  (e: 'browse'): void;
}>();
</script>

<template>
  <div class="main-content main-wrapper">
    <div class="top-bar">
      <div style="display: flex; align-items: center; gap: 12px">
        <div style="background: var(--bg-surface); border: 1px solid var(--border-subtle); border-radius: 8px; padding: 6px;">
          <FileCog :size="18" />
        </div>
        <h1>Endfield Switcher</h1>
      </div>
    </div>

    <div class="content-area" style="flex: 1; overflow-y: auto;">
      <!-- Selected Account Banner -->
      <div class="status-banner" :class="selectedAccount?.id === activeAccount?.id ? 'status-active' : ''" style="border-radius: 12px;">
        <div class="status-icon-wrapper">
          <ShieldCheck v-if="selectedAccount?.id === activeAccount?.id" :size="32" />
          <ShieldAlert v-else :size="32" />
        </div>
        <div style="flex: 1">
          <h2>{{ selectedAccount ? selectedAccount.displayName : 'Unknown Session' }}</h2>
          <p v-if="selectedAccount" style="display: flex; align-items: center; gap: 8px;">
            <span v-if="selectedAccount.id === activeAccount?.id" style="color: var(--accent-brand); font-weight: 500">Active in game</span>
            <span v-else style="color: var(--text-muted); font-weight: 500">Not active</span>
            • Last backed up: {{ selectedAccount.lastBackupTime }}
          </p>
          <p v-else>No known backups matched. Tip: Create a backup below.</p>
        </div>
      </div>

      <div class="action-grid">
        <!-- Backup Block -->
        <div class="action-card">
          <div style="display: flex; flex-direction: column; gap: 16px;">
            <div>
              <h3>
                <Plus :size="18" /> 
                Create Backup
              </h3>
              <p style="color: var(--text-secondary); font-size: 13px; line-height: 1.4; margin-top: 8px;">
                Snapshot your current login session to easily switch back to it later without entering a password.
              </p>
            </div>
          </div>
          
          <div class="input-group" style="margin-top: auto">
            <div class="input-row">
              <input
                type="text"
                :value="newNote"
                @input="emit('update:newNote', ($event.target as HTMLInputElement).value)"
                placeholder="Give this session a remark..."
                @keydown.enter="emit('backup')"
              />
              <button @click="emit('backup')" :disabled="loading" class="btn-primary">
                Save
              </button>
            </div>
          </div>
        </div>

        <!-- Launch Block -->
        <div class="action-card">
          <h3>
            <Play :size="18" /> 
            Launch Game
          </h3>
          <p style="color: var(--text-secondary); font-size: 13px; line-height: 1.4">
            Start Endfield with the currently active session. Ensure that you have specified the correct client path.
          </p>
          
          <div class="input-group" style="margin-top: auto">
            <div class="input-row">
              <input
                type="text"
                :value="exePath"
                @input="emit('update:exePath', ($event.target as HTMLInputElement).value)"
                placeholder="Game Executable Path"
                style="flex: 1"
              />
              <button @click="emit('browse')" class="btn-secondary" title="Browse File">
                <FolderSearch :size="16" />
              </button>
            </div>
            <template v-if="selectedAccount && selectedAccount.id !== activeAccount?.id">
              <button @click="emit('switchAndLaunch', selectedAccount.folderName)" :disabled="loading" class="btn-primary" style="justify-content: center; width: 100%; background-color: var(--accent-brand)">
                Switch to this Session & Launch
              </button>
            </template>
            <template v-else>
              <button @click="emit('launch')" :disabled="loading" class="btn-primary" style="justify-content: center; width: 100%">
                Launch Client
              </button>
            </template>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

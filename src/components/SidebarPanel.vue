<script setup lang="ts">
import { computed } from 'vue';
import { Edit2, Trash2, Database, Play } from 'lucide-vue-next';

export interface AccountInfo {
  id: string;
  folderName: string;
  displayName: string;
  fingerPrint: string;
  lastBackupTime: string;
}

const props = defineProps<{
  accounts: AccountInfo[];
  currentFingerprint: string;
  selectedAccountId: string | null;
  loading: boolean;
}>();

const emit = defineEmits<{
  (e: 'select', id: string): void;
  (e: 'rename', id: string, oldName: string): void;
  (e: 'delete', id: string): void;
}>();

const getAvatar = (name: string) => {
  if (!name) return '?';
  return name.charAt(0).toUpperCase();
};
</script>

<template>
  <div class="sidebar">
    <div class="sidebar-header">
      <Database :size="18" />
      <h2>Saved Sessions</h2>
    </div>
    <div class="accounts-list">
      <div v-if="accounts.length === 0" class="empty-state">No accounts backed up yet.</div>
      
      <TransitionGroup name="list">
        <div 
          v-for="acc in accounts" 
          :key="acc.id"
          :class="['account-item', { selected: selectedAccountId === acc.id }]"
          @click="emit('select', acc.id)"
        >
          <div class="acc-avatar">{{ getAvatar(acc.displayName) }}</div>
          
          <div class="acc-info">
            <div class="acc-name-row">
              <h3>{{ acc.displayName }}</h3>
            </div>
            <span class="time">{{ acc.lastBackupTime }}</span>
          </div>
          
          <div class="acc-actions">
            <!-- Indicates Active Account in Game -->
            <template v-if="currentFingerprint === acc.fingerPrint">
              <div 
                class="badge-active-small" 
                title="Currently Active"
                style="display: flex; align-items: center; justify-content: center; height: 32px; width: 32px; color: var(--accent-brand);"
              >
                <ShieldCheck :size="18" />
              </div>
            </template>

            <button 
              @click.stop="emit('rename', acc.id, acc.displayName)" 
              :disabled="loading" 
              class="btn-icon edit" 
              title="Edit Remark"
            >
              <Edit2 :size="16" />
            </button>
            <button 
              @click.stop="emit('delete', acc.id)" 
              :disabled="loading" 
              class="btn-icon delete" 
              title="Delete"
            >
              <Trash2 :size="16" />
            </button>
          </div>
        </div>
      </TransitionGroup>
    </div>
  </div>
</template>

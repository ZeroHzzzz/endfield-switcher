<script setup lang="ts">
import { ref, watch, onMounted } from 'vue';
import { X } from 'lucide-vue-next';

const props = defineProps<{
  show: boolean;
  title: string;
  type?: 'input' | 'confirm';
  initialValue?: string;
  confirmText?: string;
  cancelText?: string;
  danger?: boolean;
}>();

const emit = defineEmits<{
  (e: 'update:show', value: boolean): void;
  (e: 'confirm', value: string): void;
  (e: 'cancel'): void;
}>();

const inputValue = ref('');
const inputRef = ref<HTMLInputElement | null>(null);

watch(() => props.show, (newVal) => {
  if (newVal) {
    inputValue.value = props.initialValue || '';
    if (props.type === 'input') {
      setTimeout(() => inputRef.value?.focus(), 50);
    }
  }
});

const handleConfirm = () => {
  emit('confirm', inputValue.value);
  emit('update:show', false);
};

const handleCancel = () => {
  emit('cancel');
  emit('update:show', false);
};
</script>

<template>
  <Teleport to="body">
    <Transition name="fade">
      <div v-if="show" class="modal-backdrop" @click="handleCancel">
        <div class="modal-content" @click.stop>
          <div class="modal-header">
            <h3>{{ title }}</h3>
            <button class="btn-icon close-btn" @click="handleCancel">
              <X :size="16" />
            </button>
          </div>
          
          <div class="modal-body">
            <template v-if="type === 'input'">
              <input 
                ref="inputRef"
                type="text" 
                v-model="inputValue" 
                class="modal-input" 
                @keydown.enter="handleConfirm"
                @keydown.esc="handleCancel"
              />
            </template>
            <template v-else>
              <slot></slot>
            </template>
          </div>

          <div class="modal-footer">
            <button class="btn-secondary" @click="handleCancel">
              {{ cancelText || 'Cancel' }}
            </button>
            <button 
              :class="danger ? 'btn-danger' : 'btn-primary'" 
              @click="handleConfirm"
            >
              {{ confirmText || 'Confirm' }}
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.modal-backdrop {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  background-color: rgba(0, 0, 0, 0.7);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal-content {
  background-color: var(--bg-surface);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-lg);
  width: 400px;
  max-width: 90vw;
  box-shadow: 0 24px 48px rgba(0, 0, 0, 0.5);
  display: flex;
  flex-direction: column;
}

.modal-header {
  padding: 20px 24px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-bottom: 1px solid var(--border-subtle);
}

.modal-header h3 {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
}

.close-btn {
  color: var(--text-muted);
}
.close-btn:hover {
  color: var(--text-primary);
}

.modal-body {
  padding: 24px;
  color: var(--text-secondary);
  font-size: 14px;
  line-height: 1.5;
}

.modal-input {
  width: 100%;
  height: 40px;
  background-color: var(--bg-base);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-sm);
  padding: 0 12px;
  color: var(--text-primary);
  font-size: 14px;
  outline: none;
}
.modal-input:focus {
  border-color: var(--border-focus);
}

.modal-footer {
  padding: 16px 24px;
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  background-color: rgba(0, 0, 0, 0.2);
  border-top: 1px solid var(--border-subtle);
  border-bottom-left-radius: var(--radius-lg);
  border-bottom-right-radius: var(--radius-lg);
}

.btn-danger {
  background-color: var(--accent-danger);
  color: white;
  font-weight: 500;
  height: 36px;
  padding: 0 16px;
  border-radius: var(--radius-sm);
  display: flex;
  align-items: center;
  justify-content: center;
}
.btn-danger:hover {
  background-color: var(--accent-danger-hover);
}

.btn-secondary, .btn-primary {
  height: 36px;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease, transform 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

.fade-enter-from .modal-content,
.fade-leave-to .modal-content {
  transform: scale(0.95) translateY(10px);
}
</style>

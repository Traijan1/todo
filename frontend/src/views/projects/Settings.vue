<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { useRouter } from "vue-router";
import { useProjectStore } from "../../stores/projects";
import { useAuthStore } from "../../stores/auth";
import { storeToRefs } from "pinia";
import type { Member } from "../../api/models";

const props = defineProps<{ pid: string }>();
const router = useRouter();
const projectStore = useProjectStore();
const { user } = storeToRefs(useAuthStore());

const project = computed(() => projectStore.projects.find((p) => p.pid === props.pid));

const form = ref({ title: "", description: "", mcp_expose_comments: true });
const saving = ref(false);
const saved = ref(false);
const deleting = ref(false);

// Members
const members = ref<Member[]>([]);
const newMemberEmail = ref("");
const addingMember = ref(false);
const memberError = ref("");

const isOwner = computed(() =>
  members.value.some((m) => m.pid === user.value?.pid && m.role === "owner")
);

onMounted(async () => {
  if (!project.value) await projectStore.fetchProjects();
  if (project.value) {
    form.value = {
      title: project.value.title,
      description: project.value.description ?? "",
      mcp_expose_comments: project.value.mcp_expose_comments ?? true,
    };
  }
  members.value = await projectStore.fetchMembers(props.pid);
});

const addMember = async () => {
  if (!newMemberEmail.value.trim()) return;
  addingMember.value = true;
  memberError.value = "";
  try {
    const m = await projectStore.addMember(props.pid, newMemberEmail.value.trim());
    members.value.push(m);
    newMemberEmail.value = "";
  } catch (e: any) {
    memberError.value = e.response?.data?.description || e.response?.data?.error || "Fehler";
  } finally {
    addingMember.value = false;
  }
};

const removeMember = async (m: Member) => {
  await projectStore.removeMember(props.pid, m.pid);
  members.value = members.value.filter((x) => x.pid !== m.pid);
};

const save = async () => {
  if (!form.value.title.trim()) return;
  saving.value = true;
  saved.value = false;
  try {
    await projectStore.updateProject(props.pid, {
      title: form.value.title.trim(),
      description: form.value.description.trim() || undefined,
      mcp_expose_comments: form.value.mcp_expose_comments,
    });
    saved.value = true;
    setTimeout(() => (saved.value = false), 2000);
  } finally {
    saving.value = false;
  }
};

const confirmDelete = async () => {
  if (!confirm(`Projekt "${project.value?.title}" wirklich löschen? Alle Boards und Todos werden ebenfalls gelöscht.`)) return;
  deleting.value = true;
  try {
    await projectStore.deleteProject(props.pid);
    router.push("/");
  } finally {
    deleting.value = false;
  }
};
</script>

<template>
  <div class="max-w-2xl mx-auto w-full text-brand-text">
    <!-- Header -->
    <div class="flex items-center gap-3 mb-8">
      <RouterLink
        :to="`/projects/${pid}`"
        class="p-2 rounded-xl text-brand-text-muted/40 hover:text-brand-primary hover:bg-brand-primary/10 transition-all"
        title="Zurück zum Projekt"
      >
        <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M15 19l-7-7 7-7" />
        </svg>
      </RouterLink>
      <div>
        <p class="text-[10px] font-black uppercase tracking-widest text-brand-primary/40 mb-0.5">Projekteinstellungen</p>
        <h2 class="text-2xl font-black tracking-tight truncate">{{ project?.title || "Laden..." }}</h2>
      </div>
    </div>

    <div class="space-y-6">
      <!-- General Settings -->
      <section class="p-5 rounded-2xl bg-brand-container border border-brand-primary/10 space-y-4">
        <p class="text-[9px] font-black uppercase tracking-widest text-brand-primary/40">Allgemein</p>

        <div class="space-y-1.5">
          <label class="brand-label">Name</label>
          <input v-model="form.title" type="text" class="brand-input" placeholder="Projektname" />
        </div>

        <div class="space-y-1.5">
          <label class="brand-label">Beschreibung</label>
          <textarea v-model="form.description" class="brand-textarea" rows="3" placeholder="Kurze Projektbeschreibung (optional)" />
        </div>
      </section>

      <!-- MCP / AI Settings -->
      <section class="p-5 rounded-2xl bg-brand-container border border-brand-primary/10 space-y-4">
        <p class="text-[9px] font-black uppercase tracking-widest text-brand-primary/40">MCP / AI-Zugriff</p>

        <div class="flex items-center justify-between gap-4">
          <div>
            <p class="text-sm font-bold text-brand-text">Kommentare via MCP zugänglich</p>
            <p class="text-[11px] text-brand-text-muted/50 mt-0.5 leading-relaxed">
              Wenn aktiv, können AI-Agenten Kommentare lesen und schreiben. Deaktivieren um Kommentare vor MCP-Zugriffen zu schützen.
            </p>
          </div>
          <button
            type="button"
            class="relative w-11 h-6 rounded-full transition-all duration-200 shrink-0"
            :class="form.mcp_expose_comments ? 'bg-brand-primary' : 'bg-white/10'"
            @click="form.mcp_expose_comments = !form.mcp_expose_comments"
          >
            <span
              class="absolute top-0.5 w-5 h-5 rounded-full bg-white shadow transition-all duration-200"
              :class="form.mcp_expose_comments ? 'left-5' : 'left-0.5'"
            />
          </button>
        </div>
      </section>

      <!-- Save Button -->
      <button
        type="button"
        :disabled="saving || !form.title.trim()"
        class="w-full py-3 rounded-xl font-bold text-[11px] uppercase tracking-widest transition-all disabled:opacity-40 disabled:cursor-not-allowed flex items-center justify-center gap-2"
        :class="saved ? 'bg-emerald-500/20 text-emerald-400' : 'bg-brand-primary text-brand-container hover:bg-brand-primary/90 shadow-lg shadow-brand-primary/10'"
        @click="save"
      >
        <svg v-if="saving" class="w-3.5 h-3.5 animate-spin" fill="none" viewBox="0 0 24 24">
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"/>
          <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"/>
        </svg>
        <svg v-else-if="saved" xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M5 13l4 4L19 7" />
        </svg>
        {{ saved ? "Gespeichert!" : saving ? "Speichern..." : "Änderungen speichern" }}
      </button>

      <!-- Members -->
      <section class="p-5 rounded-2xl bg-brand-container border border-brand-primary/10 space-y-4">
        <p class="text-[9px] font-black uppercase tracking-widest text-brand-primary/40">Mitglieder</p>

        <!-- Member list -->
        <div class="space-y-2">
          <div
            v-for="m in members"
            :key="m.pid"
            class="flex items-center gap-3 py-2 px-3 rounded-xl bg-white/5"
          >
            <div class="w-7 h-7 rounded-full bg-brand-primary/20 flex items-center justify-center text-[10px] font-black text-brand-primary shrink-0">
              {{ m.name[0]?.toUpperCase() }}
            </div>
            <div class="flex-1 min-w-0">
              <p class="text-xs font-bold text-brand-text truncate">{{ m.name }}</p>
              <p class="text-[10px] text-brand-text-muted/40 truncate">{{ m.email }}</p>
            </div>
            <span
              class="text-[9px] font-black uppercase tracking-widest px-2 py-0.5 rounded-full shrink-0"
              :class="m.role === 'owner' ? 'bg-brand-primary/15 text-brand-primary' : 'bg-white/10 text-brand-text-muted/60'"
            >{{ m.role === 'owner' ? 'Owner' : 'Mitglied' }}</span>
            <button
              v-if="isOwner && m.pid !== user?.pid"
              class="text-brand-text-muted/20 hover:text-red-400 transition-all"
              title="Entfernen"
              @click="removeMember(m)"
            >
              <svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>
        </div>

        <!-- Add member (owner only) -->
        <div v-if="isOwner" class="space-y-2">
          <div class="flex gap-2">
            <input
              v-model="newMemberEmail"
              type="email"
              class="brand-input flex-1 text-xs"
              placeholder="E-Mail-Adresse"
              @keydown.enter.prevent="addMember"
            />
            <button
              type="button"
              :disabled="addingMember || !newMemberEmail.trim()"
              class="px-4 py-2 rounded-xl text-[10px] font-bold uppercase tracking-widest bg-brand-primary/20 text-brand-primary hover:bg-brand-primary/30 transition-all disabled:opacity-30 shrink-0"
              @click="addMember"
            >{{ addingMember ? '...' : 'Einladen' }}</button>
          </div>
          <p v-if="memberError" class="text-[10px] text-red-400">{{ memberError }}</p>
        </div>
      </section>

      <!-- Danger Zone -->
      <section class="p-5 rounded-2xl border border-red-500/15 bg-red-500/5 space-y-3">
        <p class="text-[9px] font-black uppercase tracking-widest text-red-400/60">Danger Zone</p>
        <div class="flex items-center justify-between gap-4">
          <div>
            <p class="text-sm font-bold text-red-300/80">Projekt löschen</p>
            <p class="text-[11px] text-brand-text-muted/40 mt-0.5">Löscht das Projekt samt aller Boards, Todos und Kommentare. Nicht rückgängig machbar.</p>
          </div>
          <button
            type="button"
            :disabled="deleting"
            class="shrink-0 px-4 py-2 rounded-xl text-[10px] font-bold uppercase tracking-widest bg-red-500/10 text-red-400 hover:bg-red-500 hover:text-white transition-all disabled:opacity-40"
            @click="confirmDelete"
          >
            {{ deleting ? "Löschen..." : "Löschen" }}
          </button>
        </div>
      </section>
    </div>
  </div>
</template>

import { defineStore } from "pinia";
import { computed, ref } from "vue";

export interface AiRequestContext {
  project_pid?: string;
  board_pid?: string;
  todo_pid?: string;
}

const normalize = (value?: string | null) => value?.trim() || undefined;

export const useAiContextStore = defineStore("ai-context", () => {
  // These are public UUID/PID values, never internal numeric database IDs.
  const projectId = ref<string | undefined>();
  const boardId = ref<string | undefined>();
  const todoId = ref<string | undefined>();

  const requestContext = computed<AiRequestContext>(() => ({
    project_pid: projectId.value,
    board_pid: boardId.value,
    todo_pid: todoId.value,
  }));

  const hasContext = computed(() => Boolean(projectId.value));

  function selectProject(nextProjectId?: string | null) {
    const normalizedProjectId = normalize(nextProjectId);
    if (projectId.value !== normalizedProjectId) {
      boardId.value = undefined;
      todoId.value = undefined;
    }
    projectId.value = normalizedProjectId;
  }

  function selectBoard(
    nextBoardId?: string | null,
    nextProjectId?: string | null,
  ) {
    if (nextProjectId !== undefined && nextProjectId !== null) {
      selectProject(nextProjectId);
    }
    boardId.value = normalize(nextBoardId);
    todoId.value = undefined;
  }

  function selectTodo(
    nextTodoId?: string | null,
    nextBoardId?: string | null,
    nextProjectId?: string | null,
  ) {
    if (nextProjectId !== undefined && nextProjectId !== null) {
      selectProject(nextProjectId);
    }
    if (nextBoardId !== undefined && nextBoardId !== null) {
      boardId.value = normalize(nextBoardId);
    }
    todoId.value = normalize(nextTodoId);
  }

  function clearTodo() {
    todoId.value = undefined;
  }

  function clearBoard() {
    boardId.value = undefined;
    todoId.value = undefined;
  }

  function clear() {
    projectId.value = undefined;
    boardId.value = undefined;
    todoId.value = undefined;
  }

  return {
    projectId,
    boardId,
    todoId,
    requestContext,
    hasContext,
    selectProject,
    selectBoard,
    selectTodo,
    clearTodo,
    clearBoard,
    clear,
  };
});

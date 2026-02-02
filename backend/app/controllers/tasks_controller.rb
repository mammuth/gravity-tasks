class TasksController < ApplicationController
  before_action :require_uid

  def index
    tasks = Task.where(uid: current_uid)

    if params[:since].present?
      since = Time.zone.parse(params[:since])
      tasks = tasks.where("updated_at > ? OR deleted_at > ?", since, since) if since
    end

    render json: tasks
  end

  def create
    attrs = ensure_list_id(task_params.to_h)
    task = Task.new(attrs.merge(uid: current_uid))
    task.save!

    render json: task, status: :created
  end

  def update
    task = Task.find_by!(id: params[:id], uid: current_uid)
    task.assign_attributes(task_params)
    task.revision = task.revision.to_i + 1
    task.save!

    render json: task
  end

  def batch
    items = params.require(:tasks)
    default_list_id = nil

    results = []
    items.each do |item|
      attrs = batch_task_params(item).to_h
      task_id = attrs.delete('id')
      task = Task.find_by(id: task_id)

      if task && task.uid != current_uid
        render json: { error: 'task_id_conflict' }, status: :conflict
        return
      end

      if attrs['list_id'].blank?
        default_list_id ||= ensure_default_list.id
        attrs['list_id'] = default_list_id
      end

      task ||= Task.new(id: task_id, uid: current_uid)
      task.assign_attributes(attrs)
      task.uid = current_uid
      task.revision = task.revision.to_i + 1 if task.persisted?
      task.save!
      results << task
    end

    render json: results
  end

  private

  def task_params
    params.require(:task).permit(:title, :status, :position, :done_at, :archived_at, :deleted_at, :list_id)
  end

  def batch_task_params(item)
    params = item.is_a?(ActionController::Parameters) ? item : ActionController::Parameters.new(item)

    params.permit(
      :id,
      :title,
      :status,
      :position,
      :done_at,
      :archived_at,
      :deleted_at,
      :list_id
    )
  end

  def ensure_list_id(attrs)
    return attrs if attrs['list_id'].present?

    attrs.merge('list_id' => ensure_default_list.id)
  end

  def ensure_default_list
    list_id = "inbox-#{current_uid}"
    List.find_or_create_by!(id: list_id, uid: current_uid) do |list|
      list.name = 'Inbox'
      list.position = 1000
      list.revision = 0
    end
  end
end

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
    task = Task.new(task_params.merge(uid: current_uid))
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

    results = []
    items.each do |item|
      attrs = batch_task_params(item).to_h
      task_id = attrs.delete('id')
      task = Task.find_by(id: task_id)

      if task && task.uid != current_uid
        render json: { error: 'task_id_conflict' }, status: :conflict
        return
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
    params.require(:task).permit(:title, :status, :position, :done_at, :archived_at, :deleted_at)
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
      :deleted_at
    )
  end
end

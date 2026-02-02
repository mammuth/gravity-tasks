class ListsController < ApplicationController
  before_action :require_uid

  def index
    lists = List.where(uid: current_uid)

    if params[:since].present?
      since = Time.zone.parse(params[:since])
      lists = lists.where("updated_at > ? OR deleted_at > ?", since, since) if since
    end

    render json: lists
  end

  def create
    list = List.new(list_params.merge(uid: current_uid))
    list.save!

    render json: list, status: :created
  end

  def update
    list = List.find_by!(id: params[:id], uid: current_uid)
    list.assign_attributes(list_params)
    list.revision = list.revision.to_i + 1
    list.save!

    cascade_delete_tasks(list) if list.deleted_at.present?

    render json: list
  end

  def batch
    items = params.require(:lists)
    results = []

    items.each do |item|
      attrs = batch_list_params(item).to_h
      list_id = attrs.delete('id')
      list = List.find_by(id: list_id)

      if list && list.uid != current_uid
        render json: { error: 'list_id_conflict' }, status: :conflict
        return
      end

      list ||= List.new(id: list_id, uid: current_uid)
      list.assign_attributes(attrs)
      list.uid = current_uid
      list.revision = list.revision.to_i + 1 if list.persisted?
      list.save!

      cascade_delete_tasks(list) if list.deleted_at.present?

      results << list
    end

    render json: results
  end

  private

  def list_params
    params.require(:list).permit(:name, :position, :deleted_at)
  end

  def batch_list_params(item)
    params = item.is_a?(ActionController::Parameters) ? item : ActionController::Parameters.new(item)

    params.permit(:id, :name, :position, :deleted_at)
  end

  def cascade_delete_tasks(list)
    Task.where(uid: current_uid, list_id: list.id).update_all(deleted_at: list.deleted_at, updated_at: Time.zone.now)
  end
end

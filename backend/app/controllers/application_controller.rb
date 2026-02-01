class ApplicationController < ActionController::API
  private

  def current_uid
    request.headers["X-UID"] || params[:uid]
  end

  def require_uid
    return if current_uid.present?

    render json: { error: "uid_required" }, status: :unauthorized
  end
end

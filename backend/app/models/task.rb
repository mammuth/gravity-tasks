class Task < ApplicationRecord
  STATUSES = %w[active done archived].freeze

  validates :uid, :title, :status, :position, :list_id, presence: true
  validates :status, inclusion: { in: STATUSES }
  validates :position, numericality: true
  validates :revision, numericality: { only_integer: true }
end

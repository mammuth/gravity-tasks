class List < ApplicationRecord
  validates :uid, :name, :position, presence: true
  validates :position, numericality: true
  validates :revision, numericality: { only_integer: true }
end
